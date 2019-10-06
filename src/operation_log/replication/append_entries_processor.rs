use crossbeam_channel::Sender;
use std::sync::{Arc, Mutex};

use crate::communication::peers::{
    AppendEntriesRequest, AppendEntriesResponse, PeerRequestHandler,
};
use crate::leadership::node_leadership_fsm::{LeaderElectionEvent, FollowerInfo};
use crate::node::state::{Node, NodeStateSaver, NodeStatus};
use crate::operation_log::OperationLog;
use crate::rsm::ReplicatedStateMachine;
use crate::Cluster;
use std::cmp::min;
use crate::leadership::watchdog::watchdog_handler::ResetLeadershipStatusWatchdog;

pub fn process_append_entries_request<Log, Rsm, Pc, Ns, Cl, Rl>(
    request: AppendEntriesRequest,
    protected_node: Arc<Mutex<Node<Log, Rsm, Pc, Ns, Cl>>>,
    leader_election_event_tx: Sender<LeaderElectionEvent>,
    leadership_status_watchdog_handler: Rl,
) -> AppendEntriesResponse
where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Pc: PeerRequestHandler,
    Ns: NodeStateSaver,
    Cl: Cluster,
    Rl : ResetLeadershipStatusWatchdog,
{
    let mut node = protected_node.lock().expect("node lock is not poisoned");

    if request.term < node.get_current_term() {
        warn!("Node {} Stale 'Append Entries Request'. Old term: {}", node.id, request.term);

        return AppendEntriesResponse {
            term: node.get_current_term(),
            success: false,
        };
    }

    //fix node status
    let should_reset_to_follower = match node.status {
        //Greater term.
        NodeStatus::Leader | NodeStatus::Candidate => {
            request.term > node.get_current_term()
        }
        //Greater term (next term) or leader changed.
        NodeStatus::Follower => {
            let should_reset_term = request.term > node.get_current_term()
                || node.current_leader_id.is_none()
                || node.current_leader_id.expect("some leader_id") != request.leader_id;

            leadership_status_watchdog_handler.reset_leadership_status_watchdog();

            should_reset_term
        }
    };

    if should_reset_to_follower {
        leader_election_event_tx
            .send(LeaderElectionEvent::ResetNodeToFollower(
                FollowerInfo{
                    term:request.term,
                    leader_id: Some(request.leader_id),
                    voted_for_id: None
                }))
            .expect("can send LeaderElectionEvent");
    }

    let previous_entry_exist =
        node.check_log_for_previous_entry(request.prev_log_term, request.prev_log_index);

    if !previous_entry_exist {
        warn!(
            "Node {} no previous entry 'Append Entries Request'. Prev term: {},Prev index: {}",
            node.id, request.prev_log_term, request.prev_log_index
        );

        return AppendEntriesResponse {
            term: node.get_current_term(),
            success: false,
        };
    }

    //add to operation log
    for entry in request.entries {
        let entry_index = entry.index;
        let append_entry_result = node.append_entry_to_log(entry);
        if let Err(err) = append_entry_result {
            error!("Append entry to Log error. Entry = {}: {}", entry_index, err);
            return AppendEntriesResponse {
                term: node.get_current_term(),
                success: false,
            };
        }
    }

    //set commit_index - lesser from leader_commit_index and log.length
    if request.leader_commit > node.get_commit_index() {
        let node_last_index = node.log.get_last_entry_index();

        let new_commit_index = min(node_last_index, request.leader_commit);
        if new_commit_index > node.get_commit_index() {
            node.set_commit_index(new_commit_index, true);
        }
    }

    AppendEntriesResponse {
        term: node.get_current_term(),
        success: true,
    }
}

use std::sync::{Arc, Mutex};
use crossbeam_channel::{Sender};

use crate::common::{LeaderConfirmationEvent};
use crate::state::{Node, NodeStatus, NodeStateSaver};
use crate::communication::peers::{AppendEntriesRequest, AppendEntriesResponse, PeerRequestHandler};
use crate::operation_log::{OperationLog};
use crate::leadership::node_leadership_status::{LeaderElectionEvent};
use crate::rsm::ReplicatedStateMachine;
use crate::Cluster;


pub fn process_append_entries_request<Log, Rsm, Pc, Ns, Cl>(request : AppendEntriesRequest,  protected_node: Arc<Mutex<Node<Log, Rsm,Pc, Ns, Cl>>>,
                                                 leader_election_event_tx: Sender<LeaderElectionEvent>,
                                                 reset_leadership_watchdog_tx: Sender<LeaderConfirmationEvent>) -> AppendEntriesResponse
    where Log: OperationLog,
          Rsm: ReplicatedStateMachine,
          Pc : PeerRequestHandler,
          Ns : NodeStateSaver,
          Cl : Cluster{
    let mut node = protected_node.lock().expect("node lock is not poisoned");

    if request.term < node.get_current_term() {
        warn!("Node {} Stale 'Append Entries Request'. Old term: {}", node.id, request.term);

        return AppendEntriesResponse { term: node.get_current_term(), success: false };
    }

    //fix node status
    match node.status {
        NodeStatus::Leader | NodeStatus::Candidate => {
            if request.term > node.get_current_term() {
                leader_election_event_tx.send(LeaderElectionEvent::ResetNodeToFollower(request.term))
                    .expect("can send LeaderElectionEvent");
            }
        },
        NodeStatus::Follower => {
            reset_leadership_watchdog_tx.send(LeaderConfirmationEvent::ResetWatchdogCounter)
                .expect("can send LeaderConfirmationEvent");
        }
    }
    node.current_leader_id = Some(request.leader_id);


    let previous_entry_exist = node.check_log_for_previous_entry(request.prev_log_term, request.prev_log_index);

    if !previous_entry_exist {
        warn!("Node {} no previous entry 'Append Entries Request'. Prev term: {},Prev index: {}", node.id, request.prev_log_term, request.prev_log_index);

        return AppendEntriesResponse { term: node.get_current_term(), success: false };
    }

    //add to operation log
    for entry in request.entries {
        let entry_index = entry.index;
        let append_entry_result = node.append_entry_to_log(entry);
        if let Err(err) = append_entry_result {
            error!("Append entry to Log error. Entry = {}: {}", entry_index, err);
            return AppendEntriesResponse { term: node.get_current_term(), success: false };
        }
    }

	//TODO bug: set_commit commit - lesser from leader_commit and log.length
    if request.leader_commit > node.get_commit_index() {
        node.set_commit_index(request.leader_commit);
    }

    AppendEntriesResponse { term: node.get_current_term(), success: true }
}


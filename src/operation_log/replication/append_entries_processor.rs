use std::sync::{Arc, Mutex};
use std::time::Duration;
use crossbeam_channel::{Sender, Receiver};

use crate::common::{LeaderConfirmationEvent};
use crate::state::{Node, NodeStatus};
use crate::communication::peers::{AppendEntriesRequest, AppendEntriesResponse};
use crate::operation_log::storage::{LogStorage};
use crate::leadership::election::{LeaderElectionEvent, ElectionNotice};

pub fn append_entries_processor<Log: Sync + Send + LogStorage>(
                                protected_node: Arc<Mutex<Node<Log>>>,
                                leader_election_event_tx : Sender<LeaderElectionEvent>,
                                append_entries_request_rx : Receiver<AppendEntriesRequest>,
                                append_entries_response_tx : Sender<AppendEntriesResponse>,
                                reset_leadership_watchdog_tx : Sender<LeaderConfirmationEvent>)
{

    loop {
        let request = append_entries_request_rx.recv().expect("can get request from append_entries_request_rx");
        let mut node = protected_node.lock().expect("node lock is not poisoned");

        trace!("Node {:?} Received 'Append Entries Request' {:?}", node.id, request);

        if request.term < node.get_current_term() {
            trace!("Node {:?} Stale 'Append Entries Request'. Old term: {:?}", node.id, request);

            //TODO change to timeout
            let resp = AppendEntriesResponse{term : node.get_current_term(), success: false};
            append_entries_response_tx.send(resp).expect("can send AppendEntriesResponse");
            continue
        }

        //fix node status
        match node.status {
            NodeStatus::Leader | NodeStatus::Candidate => {
                if request.term > node.get_current_term() {
                    let election_notice = ElectionNotice { candidate_id: request.leader_id, term: request.term };
                    leader_election_event_tx.send(LeaderElectionEvent::ResetNodeToFollower(election_notice))
                        .expect("can send LeaderElectionEvent");
                }
            },
            NodeStatus::Follower => {
                reset_leadership_watchdog_tx.send(LeaderConfirmationEvent::ResetWatchdogCounter)
                    .expect("can send LeaderConfirmationEvent");

            }
        }

        node.current_leader_id = Some(request.leader_id);

        let previous_entry_exist = node.check_log_for_previous_entry(request.prev_log_term, request.prev_log_index as usize);

        if !previous_entry_exist {
            trace!("Node {:?} no previous entry 'Append Entries Request'. Prev term: {:?},Prev index: {:?}", node.id, request.prev_log_term, request.prev_log_index);

            //TODO change to timeout
            let resp = AppendEntriesResponse{term : node.get_current_term(), success: false};
            append_entries_response_tx.send(resp).expect("can send AppendEntriesResponse");
            continue
        }

        //add to operation log
        for entry in request.entries{
            node.append_entry_to_log(entry);
        }

        //TODO change to timeout
        let resp = AppendEntriesResponse{term : node.get_current_term(), success: true};
        let send_result = append_entries_response_tx.send_timeout(resp, Duration::from_secs(1));
        trace!("Node {:?} AppendEntriesResponse: {:?}", node.id, send_result);
    }
}


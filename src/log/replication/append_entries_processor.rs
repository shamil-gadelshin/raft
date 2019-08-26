use std::sync::{Arc, Mutex};
use crossbeam_channel::{Sender, Receiver};

use crate::common::{print_event, LeaderConfirmationEvent};
use crate::state::{Node, NodeStatus};
use crate::communication::peers::{AppendEntriesRequest, AppendEntriesResponse};
use crate::log::storage::LogStorage;

pub fn append_entries_processor<Log: Sync + Send + LogStorage>(
                                mutex_node: Arc<Mutex<Node<Log>>>,
                                append_entries_request_rx : Receiver<AppendEntriesRequest>,
                                append_entries_response_tx : Sender<AppendEntriesResponse>,
                                reset_leadership_watchdog_tx : Sender<LeaderConfirmationEvent>)
{

    loop {
        let request = append_entries_request_rx.recv().expect("cannot get request from append_entries_request_rx");
        let mut node = mutex_node.lock().expect("lock is poisoned");

        print_event(format!("Node {:?} Received 'Append Entries Request' {:?}", node.id, request));

        if let NodeStatus::Leader = node.status {
            continue;
        }

        //TODO check for terms & entry index
        node.current_leader_id = Some(request.leader_id);
        reset_leadership_watchdog_tx.send(LeaderConfirmationEvent::ResetWatchdogCounter).expect("cannot send LeaderConfirmationEvent");

        //TODO change to timeout
        let resp = AppendEntriesResponse{term : node.current_term, success: true};
        append_entries_response_tx.send(resp).expect("cannot send AppendEntriesResponse");
    }
}


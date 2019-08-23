use std::sync::{Arc, Mutex};
use crossbeam_channel::{Sender, Receiver};

use crate::core::*;
use crate::communication::peers::AppendEntriesRequest;

pub fn append_entries_processor(
                                mutex_node: Arc<Mutex<Node>>,
                                append_entries_request_rx : Receiver<AppendEntriesRequest>,
                                reset_leadership_watchdog_tx : Sender<LeaderConfirmationEvent>)
{

    loop {
        let request_result = append_entries_request_rx.recv();
        let request = request_result.unwrap(); //TODO
        let node = mutex_node.lock().expect("lock is poisoned");

        print_event(format!("Node {:?} Received 'Append Entries Request' {:?}", node.id, request));

        if let NodeStatus::Leader = node.status {
            continue;
        }

        reset_leadership_watchdog_tx.send(LeaderConfirmationEvent::ResetWatchdogCounter).expect("cannot send LeaderConfirmationEvent");
    }
}


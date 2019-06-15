use std::sync::{Arc, Mutex};
use std::time::Duration;


use crate::leadership::core::*; //TODO change project structure
use crate::leadership::communication::*; //TODO change project structure

pub fn send_append_entries(mutex_node: Arc<Mutex<Node>>,
                           peers : Vec<u64>,
                           communicator : InProcNodeCommunicator
                           //                    watchdog_event_rx : Receiver<LeaderElectedEvent>
) {

    /*
    lock leader-mutex
    */



    loop {
        let heartbeat_timeout = crossbeam_channel::after(leader_heartbeat_duration_ms());
        select!(
            recv(heartbeat_timeout) -> _  => {},
//            recv(watchdog_event_rx) -> _ => {
//                let node = mutex_node.lock().expect("lock is poisoned");
//                print_event(format!("Node {:?} Received reset watchdog ", node.id));
//                continue
//            },
            //TODO : notify leadership
        );

        let node = mutex_node.lock().expect("lock is poisoned");

        let peers_list_copy = peers.clone();
        if let NodeStatus::Leader = node.status {
            let append_entries_heartbeat = AppendEntriesRequest { term: node.current_term, leader_id: node.id };
            for peer_id in peers_list_copy {
                communicator.send_append_entries_request(peer_id, append_entries_heartbeat);
            }

            print_event(format!("Node {:?} Send 'Append Entries Request(empty)'.", node.id));

        }
    }
}


fn leader_heartbeat_duration_ms() -> Duration{
    Duration::from_millis(1000)
}

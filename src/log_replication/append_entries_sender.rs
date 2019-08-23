use std::sync::{Arc, Mutex};
use std::time::Duration;
use crossbeam_channel::{Sender, Receiver};

use crate::core::*;
use crate::communication::*;
use crate::runner::NodeConfiguration;


//TODO remove clone-values
pub fn send_append_entries(protected_node: Arc<Mutex<Node>>,
                           cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
//                           change_server_membership_rx :  Receiver<AddServerRequest>,
                           communicator : InProcNodeCommunicator
) {
    loop {
        let heartbeat_timeout = crossbeam_channel::after(leader_heartbeat_duration_ms());
        select!(
            recv(heartbeat_timeout) -> _  => {
                send_heart_beat(protected_node.clone(), cluster_configuration.clone(), communicator.clone())
                },
//            recv(watchdog_event_rx) -> _ => {
//                let node = mutex_node.lock().expect("lock is poisoned");
//                print_event(format!("Node {:?} Received reset watchdog ", node.id));
//                continue
//            },
            //TODO : notify leadership
        );


    }
}

fn send_heart_beat(protected_node : Arc<Mutex<Node>>,
                   cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
                   communicator : InProcNodeCommunicator) {
    let (node_id, node_status, node_term)  = {
        let node = protected_node.lock().expect("node lock is poisoned");

        (node.id, node.status, node.current_term)
    };

    if let NodeStatus::Leader = node_status {
        let peers_list_copy =  {
            let cluster = cluster_configuration.lock().expect("cluster lock is poisoned");

            cluster.get_peers(node_id)
        };

        let append_entries_heartbeat = AppendEntriesRequest { term: node_term, leader_id: node_id };
        for peer_id in peers_list_copy {
            communicator.send_append_entries_request(peer_id, append_entries_heartbeat);
        }

        print_event(format!("Node {:?} Send 'Append Entries Request(empty)'.", node_id));
    }
}


fn leader_heartbeat_duration_ms() -> Duration{
    Duration::from_millis(1000)
}

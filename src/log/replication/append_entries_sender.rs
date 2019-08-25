use std::sync::{Arc, Mutex};
use std::time::Duration;
use crossbeam_channel::{Receiver};

use crate::core::{Node, NodeStatus, print_event};
use crate::communication::peers::{InProcNodeCommunicator, AppendEntriesRequest};
use crate::communication::client::{AddServerRequest};
use crate::configuration::cluster::{ClusterConfiguration};
use crate::log::storage::LogStorage;

//TODO remove clone-values
pub fn send_append_entries<Log: Sync + Send + LogStorage>(protected_node: Arc<Mutex<Node<Log>>>,
                                                          cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
                                                          change_server_membership_rx :  Receiver<AddServerRequest>,
                                                          communicator : InProcNodeCommunicator
) {
    loop {
        let heartbeat_timeout = crossbeam_channel::after(leader_heartbeat_duration_ms());
        select!(
            recv(heartbeat_timeout) -> _  => {
                send_heartbeat(protected_node.clone(), cluster_configuration.clone(), communicator.clone())
                },
            recv(change_server_membership_rx) -> req => {
                send_change_membership(req.unwrap())
            },
        );


    }
}

fn send_change_membership(request : AddServerRequest) {
    print_event(format!("Node {:?} Send 'Append Entries Request(change membership)'.", request));
}

fn send_heartbeat<Log: Sync + Send + LogStorage>(protected_node : Arc<Mutex<Node<Log>>>,
                                                 cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
                                                 communicator : InProcNodeCommunicator) {
    let (node_id, node_status)  = {
        let node = protected_node.lock().expect("node lock is poisoned");

        (node.id, node.status)
    };

    if let NodeStatus::Leader = node_status {
        let peers_list_copy =  {
            let cluster = cluster_configuration.lock().expect("cluster lock is poisoned");

            cluster.get_peers(node_id)
        };

        let append_entries_heartbeat_template = create_empty_append_entry_request(protected_node);

        print_event(format!("Node {:?} Send 'Append Entries Request(empty)'.", node_id));

        //TODO communicator timeout handling
        //TODO rayon parallel-foreach
        for peer_id in peers_list_copy {
            communicator.send_append_entries_request(peer_id, append_entries_heartbeat_template.clone());
        }
    }
}

fn create_empty_append_entry_request<Log: Sync + Send + LogStorage>(protected_node : Arc<Mutex<Node<Log>>>) -> AppendEntriesRequest {
    let node = protected_node.lock().expect("node lock is poisoned");

    let append_entries_heartbeat = AppendEntriesRequest {
        term: node.current_term,
        leader_id: node.id,
        prev_log_term : node.get_last_entry_term(),
        prev_log_index : node.get_last_entry_index(),
        leader_commit : 0, //TODO fsm
        entries : Vec::new()
    };

    append_entries_heartbeat
}

fn leader_heartbeat_duration_ms() -> Duration{
    Duration::from_millis(1000)
}

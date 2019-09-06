use std::sync::{Arc, Mutex};
use std::time::Duration;
use crossbeam_channel::{Receiver};


use crate::state::{Node, NodeStatus, AppendEntriesRequestType};
use crate::communication::peers::{InProcNodeCommunicator, AppendEntriesRequest};
use crate::communication::peer_notifier::notify_peers;
use crate::configuration::cluster::{ClusterConfiguration};
use crate::operation_log::LogStorage;
use crate::fsm::Fsm;

//TODO remove clone-values
//TODO park-unpark the thread
pub fn send_heartbeat_append_entries<Log: Sync + Send + LogStorage, FsmT:  Sync + Send + Fsm>(protected_node: Arc<Mutex<Node<Log, FsmT>>>,
                                                                    cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
                                                                    leader_initial_heartbeat_rx : Receiver<bool>,
                                                                    communicator : InProcNodeCommunicator
) {
    loop {
        let heartbeat_timeout = crossbeam_channel::after(leader_heartbeat_duration_ms());
        select!(
            recv(heartbeat_timeout) -> _  => {
                send_heartbeat(protected_node.clone(), cluster_configuration.clone(), &communicator)
                },
            recv(leader_initial_heartbeat_rx) -> _  => {
                trace!("Sending initial heartbeat...");
                send_heartbeat(protected_node.clone(), cluster_configuration.clone(), &communicator)
                },
        );
    }
}

fn send_heartbeat<Log: Sync + Send + LogStorage, FsmT:  Sync + Send + Fsm>(protected_node : Arc<Mutex<Node<Log, FsmT>>>,
                                                 cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
                                                 communicator : &InProcNodeCommunicator) {
    let node = protected_node.lock().expect("node lock is not poisoned");

    if let NodeStatus::Leader = node.status {
        let cluster = cluster_configuration.lock().expect("cluster lock is not poisoned");

        let peers_list_copy = cluster.get_peers(node.id);

        let append_entries_heartbeat=
            node.create_append_entry_request(AppendEntriesRequestType::Heartbeat);

        trace!("Node {:?} Send 'empty Append Entries Request(heartbeat)'.", node.id);

        let requester = |dest_node_id: u64, req: AppendEntriesRequest| communicator.send_append_entries_request(dest_node_id, req);
        let result = notify_peers(append_entries_heartbeat, node.id,peers_list_copy, None, requester);

        if result.is_err(){
            error!("Node {:?} Send heartbeat failed: {}", node.id, result.unwrap_err().description())
        }
    }
}

fn leader_heartbeat_duration_ms() -> Duration{
    Duration::from_millis(1000)
}

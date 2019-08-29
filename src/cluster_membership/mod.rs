use std::sync::{Arc, Mutex};

use crossbeam_channel::{Sender, Receiver};

use crate::state::{Node, NodeStatus};
use crate::communication::client::{AddServerRequest,AddServerResponse, ChangeMembershipResponseStatus};
use crate::configuration::cluster::{ClusterConfiguration};
use crate::operation_log::storage::LogStorage;

//TODO remove
pub fn change_membership<Log: Sync + Send + LogStorage>(mutex_node: Arc<Mutex<Node<Log>>>,
                                                        cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
                                                        client_add_server_request_rx : Receiver<AddServerRequest>,
                                                        client_add_server_response_tx : Sender<AddServerResponse>,
                                                        internal_add_server_channel_tx : Sender<AddServerRequest>) {
    loop {
        let request = client_add_server_request_rx.recv().expect("cannot get request from client_add_server_request_rx");

        let (node_id, node_status, current_leader_id) = {
            let node = mutex_node.lock().expect("node lock is poisoned");

            (node.id, node.status, node.current_leader_id)
        };

        if let NodeStatus::Leader = node_status{
            let add_server_response = AddServerResponse{status: ChangeMembershipResponseStatus::Ok, current_leader:current_leader_id};

            info!("Node {:?} Received 'Add Server Request (Node {:?})' {:?}", node_id, request.new_server, request);

            let mut cluster = cluster_configuration.lock().expect("cluster lock is not poisoned");

            cluster.add_peer(request.new_server);

            internal_add_server_channel_tx.send(request).expect("can send request to internal_add_server_channel_tx");

            client_add_server_response_tx.send(add_server_response).expect("can send client_add_server_response");

        } else {
            let add_server_response = AddServerResponse{status: ChangeMembershipResponseStatus::NotLeader, current_leader:current_leader_id};

            client_add_server_response_tx.send(add_server_response).expect("can send response client_add_server");
        }
    }
}

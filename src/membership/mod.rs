use std::sync::{Arc, Mutex};

use crossbeam_channel::{Sender, Receiver};

use crate::core::*;
use crate::communication::client::{AddServerRequest,AddServerResponse, ChangeMembershipResponseStatus};
use crate::configuration::cluster::{ClusterConfiguration};

//TODO join client request handler
pub fn change_membership(mutex_node: Arc<Mutex<Node>>,
                         cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
                         client_add_server_request_rx : Receiver<AddServerRequest>,
                         client_add_server_response_tx : Sender<AddServerResponse>,
                         internal_add_server_channel_tx : Sender<AddServerRequest>) {
    loop {
        let add_server_request_result = client_add_server_request_rx.recv();
        let request = add_server_request_result.unwrap(); //TODO

        let (node_id, node_status, current_leader_id) = {
            let node = mutex_node.lock().expect("lock is poisoned");

            (node.id, node.status, node.current_leader_id)
        };

        if let NodeStatus::Leader = node_status{
            let add_server_response = AddServerResponse{status: ChangeMembershipResponseStatus::Ok, current_leader:current_leader_id};

            client_add_server_response_tx.send(add_server_response).expect("cannot send client_add_server_response");

            print_event(format!("Node {:?} Received 'Add Server Request (Node {:?})' {:?}", node_id, request.new_server, request));

            let mut cluster = cluster_configuration.lock().expect("cluster lock is poisoned");

            cluster.add_peer(request.new_server);

            internal_add_server_channel_tx.send(request).unwrap(); //TODO error handling

        } else {
            let add_server_response = AddServerResponse{status: ChangeMembershipResponseStatus::NotLeader, current_leader:current_leader_id};

            client_add_server_response_tx.send(add_server_response).expect("cannot send client_add_server_response");
        }
    }
}

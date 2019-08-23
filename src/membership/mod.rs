use std::time::{Duration};
use rand::Rng;
use std::sync::{Arc, Mutex};

use crossbeam_channel::{Sender, Receiver};

use crate::core::*;

use crate::leadership::election::{LeaderElectionEvent, ElectionNotice};


//TODO join client request handler
pub fn change_membership(mutex_node: Arc<Mutex<Node>>,
                         cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
                         add_server_channel_rx : Receiver<AddServerRequest>) {
    loop {
        let add_server_request_result = add_server_channel_rx.recv();
        let request = add_server_request_result.unwrap(); //TODO
        let node = mutex_node.lock().expect("lock is poisoned");

        print_event(format!("Node {:?} Received 'Add Server Request (Node {:?})' {:?}", node.id, request.new_server, request));

        let mut cluster = cluster_configuration.lock().expect("cluster lock is poisoned");

        cluster.add_peer(request.new_server);
    }
}
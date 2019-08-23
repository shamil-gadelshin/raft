use std::thread;
use std::sync::{Arc, Mutex};

#[macro_use]
extern crate crossbeam_channel;
extern crate chrono;


mod core;
mod node_runner;
mod leadership;
mod communication;
mod log_replication;
mod membership;
mod client_requests;

use client_requests::ClientRequestHandler;
use communication::{InProcNodeCommunicator};
use crate::core::{ClusterConfiguration, AddServerRequest};
use std::time::Duration;
use std::collections::HashMap;

fn main() {
    let node_ids = vec![1, 2];
    let new_node_id = 3;

    let main_cluster_configuration = ClusterConfiguration::new(node_ids);

    let mut communicator = InProcNodeCommunicator::new(main_cluster_configuration.get_all());
    communicator.add_node_communication(new_node_id);

    let mut client_handlers : HashMap<u64, ClientRequestHandler> = HashMap::new();
    for node_id in main_cluster_configuration.get_all() {
        let protected_cluster_config = Arc::new(Mutex::new(ClusterConfiguration::new(main_cluster_configuration.get_all())));

        let client_request_handler = ClientRequestHandler::new();
        let config = node_runner::NodeConfiguration {
            node_id,
            cluster_configuration: protected_cluster_config.clone(),
            communicator: communicator.clone(),
            client_request_handler: client_request_handler.clone(),
        };
        thread::spawn(move || node_runner::start_node(config));

        client_handlers.insert(node_id, client_request_handler);
    }

    let protected_cluster_config = Arc::new(Mutex::new(ClusterConfiguration::new(main_cluster_configuration.get_all())));
    run_add_server_thread_with_delay(communicator.clone(), protected_cluster_config,
                                     client_handlers,
                                     new_node_id);

    //thread::park(); //TODO -  to join
    thread::sleep(Duration::from_secs(10000));
}

fn run_add_server_thread_with_delay(communicator : communication::InProcNodeCommunicator,
                                    protected_cluster_config : Arc<Mutex<ClusterConfiguration>>,
                                    client_handlers : HashMap<u64, ClientRequestHandler>,
                                    new_node_id : u64) {

    let new_server_config;
    {
        let mut cluster_config = protected_cluster_config.lock().expect("cluster config lock is poisoned");
        cluster_config.add_peer(new_node_id);

        // *** add new server
        new_server_config = node_runner::NodeConfiguration {
            node_id: new_node_id,
            cluster_configuration: protected_cluster_config.clone(),
            communicator: communicator.clone(),
            client_request_handler : ClientRequestHandler::new()
        };
    }
    let timeout = crossbeam_channel::after(Duration::new(3,0));
    select!(
            recv(timeout) -> _  => {},
        );
    thread::spawn(move || node_runner::start_node(new_server_config));

    let request = AddServerRequest{new_server : new_node_id};
    for kv in client_handlers {
        let (k,v) = kv;

        let resp = v.add_server(request);

        core::print_event(format!("Add server request sent for NodeId = {:?}. Response = {:?}", k, resp));
    }
}
/*
TODO:
- logging
- crossbeam?rayon threading
- futures?
- tests
- speed & memory profiling
- identity - libp2p
- generic identity?
- tarpc
- consider replacing mutex with cas for nodes
- RW-lock for communicator
- check channels overflow
- check channels bounded-unbounded types

Features:
- log replication
    .memory snapshot
    .file snapshot
    .persist server's current term and vote
- membership changes
    .add server
    .change quorum size
    .remove server(shutdown self)
- client_requests support
    .server api
    .separate client_requests
- library crate

Done:
- leader election
- channel communication
- modules create
*/

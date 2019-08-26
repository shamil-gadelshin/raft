use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::HashMap;

#[macro_use]
extern crate crossbeam_channel;
extern crate chrono;

mod common;
mod node_runner;
mod leadership;
mod communication;
mod log;
mod membership;
mod configuration;
mod state;

use communication::client::{AddServerRequest,ClientRequestHandler};
use communication::peers::{InProcNodeCommunicator};
use crate::log::storage::{MemoryLogStorage};
use crate::configuration::cluster::ClusterConfiguration;
use crate::configuration::node::NodeConfiguration;

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
        let config = NodeConfiguration {
            node_id,
            cluster_configuration: protected_cluster_config.clone(),
            peer_communicator: communicator.clone(),
            client_request_handler: client_request_handler.clone(),
        };
        thread::spawn(move || node_runner::start_node(config,  MemoryLogStorage::new()));

        client_handlers.insert(node_id, client_request_handler);
    }

    let protected_cluster_config = Arc::new(Mutex::new(ClusterConfiguration::new(main_cluster_configuration.get_all())));
    run_add_server_thread_with_delay(communicator.clone(), protected_cluster_config,
                                     client_handlers,
                                     new_node_id);
    //TODO -  change to 'join to the node thread'
    thread::sleep(Duration::from_secs(86000 * 1000));
}

fn run_add_server_thread_with_delay(communicator : InProcNodeCommunicator,
                                    protected_cluster_config : Arc<Mutex<ClusterConfiguration>>,
                                    client_handlers : HashMap<u64, ClientRequestHandler>,
                                    new_node_id : u64) {

    let new_server_config;
    {
        let mut cluster_config = protected_cluster_config.lock().expect("cluster config lock is poisoned");
        cluster_config.add_peer(new_node_id);

        // *** add new server
        new_server_config = NodeConfiguration {
            node_id: new_node_id,
            cluster_configuration: protected_cluster_config.clone(),
            peer_communicator: communicator.clone(),
            client_request_handler : ClientRequestHandler::new(),
        };
    }
    let timeout = crossbeam_channel::after(Duration::new(3,0));
    select!(
            recv(timeout) -> _  => {},
        );
    thread::spawn(move || node_runner::start_node(new_server_config,MemoryLogStorage::new()));

    let request = AddServerRequest{new_server : new_node_id};
    for kv in client_handlers {
        let (k,v) = kv;

        let resp = v.add_server(request);

        common::print_event(format!("Add server request sent for NodeId = {:?}. Response = {:?}", k, resp));
    }
}


/*
TODO: Features:
- investigate
   .futures
   .check channels bounded-unbounded types
   .check channels overflows
   .election trait?
   .extract communicator trait
   .project timeouts
- fsm support
- identity
    .generic
    .libp2p
- communication
    .client_requests support
        .server api
        .separate client_requests
    .libp2p
    .tarpc
    .grpc
    .change client and node response patterns (after committing to the log)
- log replication
    .memory snapshot
    .file snapshot
    .persist server's current term and vote and cluster configuration (state persistence)
    .response to the client after majority of the servers responses
    .log forcing from the leader
        .empty (heartbeat) AppendEntries on node's current log index evaluating
    .support max AppendEntries size parameter
- membership changes
    .change quorum size
    .remove server(shutdown self)
- system events logging
    .introduce logging system (remove print_event())
    .increase log coverage
- error handling
    .error style
    .communication timeouts
    .investigate channels faults
- project structure
    .library crate
    .separate client?
    .exe-project
    .raft vs infrastructure module(code) separation
- release
    .consensus
    .configuration
        .cmd params
        .file (toml?)
- debug
    .tests
    .cases
    .conditional compilation
- user-friendliness
    .readme.md
    .documentation
    .license
- optimization
    .rayon on nodes requests
    .speed & memory profiling
    .consider replacing mutex with cas (or RW-lock) for nodes
    .RW-lock for communicator
    .optional abort channel for peers notifier (election) (abort_election_event_rx in notify_peers fn)

Future Features:
- transfer leadership


Done:
- leader election
- channel communication
- modules create
- membership changes
    .add server
*/

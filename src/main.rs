#[macro_use] extern crate log;
extern crate env_logger;

#[macro_use]
extern crate crossbeam_channel;
extern crate chrono;

mod common;
mod node_runner;
mod leadership;
mod communication;
mod operation_log;
mod cluster_membership;
mod configuration;
mod state;
mod fsm;
mod request_handler;

use communication::client::{AddServerRequest, InProcClientCommunicator};
use communication::peers::{InProcNodeCommunicator};
use crate::operation_log::storage::{MemoryLogStorage};
use crate::configuration::cluster::ClusterConfiguration;
use crate::configuration::node::NodeConfiguration;

use chrono::prelude::{DateTime, Local};

use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::HashMap;
use std::io::Write;

fn main() {
    env_logger::builder()
        .format(|buf, record| {
            let now: DateTime<Local> = Local::now();
            writeln!(buf, "{:5}: {} - {}", record.level(),now.format("%H:%M:%S.%3f").to_string(), record.args())
        })
        .init();

    let node_ids = vec![1, 2];
    let new_node_id = 3;

    let main_cluster_configuration = ClusterConfiguration::new(node_ids);

    let mut communicator = InProcNodeCommunicator::new(main_cluster_configuration.get_all());
    communicator.add_node_communication(new_node_id);

    let mut client_handlers : HashMap<u64, InProcClientCommunicator> = HashMap::new();
    for node_id in main_cluster_configuration.get_all() {
        let protected_cluster_config = Arc::new(Mutex::new(ClusterConfiguration::new(main_cluster_configuration.get_all())));

        let client_request_handler = InProcClientCommunicator::new();
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
                                    client_handlers : HashMap<u64, InProcClientCommunicator>,
                                    new_node_id : u64) {
return;

    let new_server_config;
    {
        let mut cluster_config = protected_cluster_config.lock().expect("cluster config lock is poisoned");
        cluster_config.add_peer(new_node_id);

        // *** add new server
        new_server_config = NodeConfiguration {
            node_id: new_node_id,
            cluster_configuration: protected_cluster_config.clone(),
            peer_communicator: communicator.clone(),
            client_request_handler : InProcClientCommunicator::new(),
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

        info!("Add server request sent for NodeId = {:?}. Response = {:?}", k, resp);
    }
}

/*
TODO: Features:
- stability
    .crossbeam recv, send - parse result
    .check channels bounded-unbounded types
    .check channels overflows
    .project timeouts
    .investigate channels faults
- investigate
   .'static in the 'Log : LogStorage + Sync + Send+ 'static'
   .futures
   .election trait?
   .extract communicator trait
   .rebuild raft election as fsm
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
    .change client and node response patterns (after committing to the operation_log)
- operation_log replication
    .memory snapshot
    .file snapshot
    .persist server's current term and vote and cluster configuration (state persistence)
    .response to the client after majority of the servers responses
    .operation_log forcing from the leader
        .empty (heartbeat) AppendEntries on node's current operation_log index evaluating
    .support max AppendEntries size parameter & max AppendEntries number
- cluster membership changes
    .change quorum size
    .remove server(shutdown self)
- system events logging
    .remove requests from log messages
    .increase log coverage
- error handling
    .error style
    .communication timeouts
- project structure
    .library crate
    .separate client?
    .exe-project
        .remove node_id from log
    .raft vs infrastructure module(code) separation
    .dev-dependencies
        .env-logger
- code style
    .investigate & remove use crate::
    .introduce aliases (Arc<Mutex<Node<Log>>>)
    .enforce code line length limit
    .rustfmt
- release
    .consensus
    .configuration
        .cmd params
        .file (toml?)
- debug
    .tests
    .cases
        .cases description in file
        .election
            .invalid term (stale)
            .newer term - convert to follower
                .leader
                .candidate
    .conditional compilation
- user-friendliness
    .readme.md
    .documentation
        .add the top of the main file: #![warn(missing_docs, unsafe_code)]
    .license
    .Rust API Guidelines
- optimization
    .rayon on nodes requests
    .speed & memory profiling
    .consider replacing mutex with cas (or RW-lock) for nodes
    .RW-lock for communicator
    .optional abort channel for peers notifier (election) (abort_election_event_rx in notify_peers fn)
    .migrate Mutex to parking_lot implementation

Future Features:
- transfer leadership
- log compaction



Done:
- leader election
- channel communication
- modules create
- cluster_membership changes
    .add server
- system events logging
    .introduce logging system (remove print_event())
*/

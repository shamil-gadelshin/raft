#[macro_use] extern crate log;
extern crate env_logger;

extern crate chrono;
#[macro_use] extern crate crossbeam_channel;

mod common;
mod leadership;
mod communication;
mod operation_log;
mod configuration;
mod state;
mod fsm;
mod request_handler;
mod workers;
mod errors;


use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::HashMap;
use std::io::Write;
use std::thread::JoinHandle;

use chrono::prelude::{DateTime, Local};

use communication::client::{AddServerRequest, InProcClientCommunicator};
use communication::peers::{InProcNodeCommunicator};
use operation_log::storage::{MemoryLogStorage};
use configuration::cluster::ClusterConfiguration;
use configuration::node::NodeConfiguration;
use std::thread;
use crate::communication::client::NewDataRequest;


fn main() {
    init_logger();

    let node_ids = vec![1, 2];
    let new_node_id = 3;
    let communication_timeout = Duration::from_millis(500);
    let main_cluster_configuration = ClusterConfiguration::new(node_ids);

    let mut communicator = InProcNodeCommunicator::new(main_cluster_configuration.get_all(), communication_timeout);
    communicator.add_node_communication(new_node_id);

    let mut client_handlers : HashMap<u64, InProcClientCommunicator> = HashMap::new();
	let mut node_threads = Vec::new();
    for node_id in main_cluster_configuration.get_all() {
        let protected_cluster_config = Arc::new(Mutex::new(ClusterConfiguration::new(main_cluster_configuration.get_all())));

        let client_request_handler = InProcClientCommunicator::new(node_id, communication_timeout);
        let config = NodeConfiguration {
            node_id,
            cluster_configuration: protected_cluster_config.clone(),
            peer_communicator: communicator.clone(),
            client_communicator: client_request_handler.clone(),
        };
        let thread_handle = workers::node_main_process::run_thread(config, MemoryLogStorage::new());
		node_threads.push(thread_handle);

        client_handlers.insert(node_id, client_request_handler);
    }

    let protected_cluster_config = Arc::new(Mutex::new(ClusterConfiguration::new(main_cluster_configuration.get_all())));
    let thread_handle = run_add_server_thread_with_delay(communicator.clone(), protected_cluster_config,
                                     client_handlers,
                                     new_node_id);

	node_threads.push(thread_handle);

	for node_thread in node_threads {
		let thread = node_thread.join();
        if thread.is_err(){
            panic!("worker panicked!")
        }
	}
}

fn init_logger() {
    env_logger::builder()
        .format(|buf, record| {
            let now: DateTime<Local> = Local::now();
            writeln!(buf, "{:5}: {} - {}", record.level(), now.format("%H:%M:%S.%3f").to_string(), record.args())
        })
        .init();
}

fn run_add_server_thread_with_delay(communicator : InProcNodeCommunicator,
                                    protected_cluster_config : Arc<Mutex<ClusterConfiguration>>,
                                    client_handlers : HashMap<u64, InProcClientCommunicator>,
                                    new_node_id : u64) -> JoinHandle<()>{
//return;

    let communication_timeout = Duration::from_millis(500);

    thread::sleep(Duration::from_secs(3));

    let new_server_config;
    {
        new_server_config = NodeConfiguration {
            node_id: new_node_id,
            cluster_configuration: protected_cluster_config.clone(),
            peer_communicator: communicator.clone(),
            client_communicator: InProcClientCommunicator::new(new_node_id,communication_timeout),
        };
    }

    let thread_handle = workers::node_main_process::run_thread(new_server_config, MemoryLogStorage::new());

    let add_server_request = AddServerRequest{new_server : new_node_id};
    for kv in client_handlers.clone() {
        let (k,v) = kv;

        let resp = v.add_server(add_server_request);

        info!("Add server request sent for NodeId = {:?}. Response = {:?}", k, resp);
    }

    thread::sleep(Duration::from_secs(2));

    let bytes = "first data".as_bytes();
    let new_data_request = NewDataRequest{data : Arc::new(bytes)};
    for kv in client_handlers {
        let (k,v) = kv;

        let resp = v.new_data(new_data_request.clone());

        info!("New Data request sent for NodeId = {:?}. Response = {:?}", k, resp);
    }

	thread_handle
}

/*
TODO: Features:

Main algorithm:
- fsm
    .persistent
        .load on start
- election
    .check log on votes
- operation_log replication
    .file snapshot
        .load on start
    .persist server's current term and vote and cluster configuration (state persistence)
    .response to the client after majority of the servers responses
    .operation_log forcing from the leader
        .empty (heartbeat) AppendEntries on node's current operation_log index evaluating
    .support max AppendEntries size parameter & max AppendEntries number
    .not commit entry if quorum fails (clear entry?)
- cluster membership changes
    .change quorum size
    .remove server(shutdown self)
- client RPC
    .sessions - for duplicate request handling
    .read query
Details:
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
   .worker thread structure: all peer-request thread, client-thread?
   .rebuild raft election as fsm: implement explicit transitions causes (received AppendEntryRequest, HeartbeatWaitingTimeExpired, etc)
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
            .incorrect leader sends heartbeats
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
- fsm support
- leader election
- channel communication
- modules create
- cluster_membership changes
    .add server
- system events logging
    .introduce logging system (remove print_event())
        .response to the client after majority of the servers responses
- operation_log
    .memory snapshot
    .empty (heartbeat) AppendEntries on node's current operation_log index evaluating
*/

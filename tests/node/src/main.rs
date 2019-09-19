#[macro_use] extern crate log;
extern crate env_logger;
extern crate chrono;
extern crate crossbeam_channel;

use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::io::Write;

use chrono::prelude::{DateTime, Local};

extern crate raft;
extern crate raft_modules;

use raft::{NodeState, NodeTimings};
use raft::ClusterConfiguration;
use raft::NodeConfiguration;

use raft_modules::{MemoryRsm, RandomizedElectionTimer, MockNodeStateSaver};
use raft_modules::MemoryOperationLog;
use raft_modules::{InProcPeerCommunicator};
use raft_modules::NetworkClientCommunicator;


fn init_logger() {
    env_logger::builder()
        .format(|buf, record| {
            let now: DateTime<Local> = Local::now();
            writeln!(buf, "{:5}: {} - {}", record.level(), now.format("%H:%M:%S.%3f").to_string(), record.args())
        })
        .init();
}



fn main() {
    init_logger();

    let node_id = 1;

    info!("Server started");

    let cluster_configuration = ClusterConfiguration::new(vec![node_id]);
    let protected_cluster_config = Arc::new(Mutex::new(ClusterConfiguration::new(cluster_configuration.get_all_nodes())));
    let communication_timeout = Duration::from_millis(500);
    let client_request_handler = NetworkClientCommunicator::new(get_address(node_id), node_id, communication_timeout, true);

    let node_config = NodeConfiguration {
        node_state: NodeState {
            node_id,
            current_term: 0,
            vote_for_id: None
        },
        cluster_configuration: protected_cluster_config.clone(),
        peer_communicator: InProcPeerCommunicator::new(cluster_configuration.get_all_nodes(), communication_timeout),
        client_communicator: client_request_handler.clone(),
        election_timer: RandomizedElectionTimer::new(1000, 4000),
        timings: NodeTimings::default(),
        operation_log: MemoryOperationLog::new(protected_cluster_config.clone()),
        rsm: MemoryRsm::default(),
        state_saver: MockNodeStateSaver::default()
    };

    let node_worker = raft::start_node(node_config
                                       );

    let thread = node_worker.join_handle.join();
    if thread.is_err() {
        panic!("worker panicked!")
    }
}


fn get_address(node_id : u64) -> String{
    format!("127.0.0.1:{}", 50000 + node_id)
}


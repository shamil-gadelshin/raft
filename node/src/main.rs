#[macro_use] extern crate log;
extern crate env_logger;
extern crate chrono;
extern crate crossbeam_channel;

use std::time::Duration;
use std::io::Write;
use std::collections::HashMap;

use chrono::prelude::{DateTime, Local};

extern crate raft;
extern crate raft_modules;

use raft::{NodeState, NodeLimits};
use raft::NodeConfiguration;

use raft_modules::{MemoryRsm, RandomizedElectionTimer, MockNodeStateSaver, ClusterConfiguration};
use raft_modules::{NetworkPeerCommunicator, StaticTableServiceDiscovery};
use raft_modules::MemoryOperationLog;
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
    let all_nodes = vec![node_id];
    let cluster_configuration = ClusterConfiguration::new(all_nodes.clone());

    let client_request_handler = NetworkClientCommunicator::new(get_client_requests_address(node_id), node_id, get_communication_timeout(), true);
    let peer_request_handler = get_network_peer_communicator(node_id);

    let node_config = NodeConfiguration {
        node_state: NodeState {
            node_id,
            current_term: 0,
            vote_for_id: None
        },
        cluster_configuration: cluster_configuration.clone(),
        peer_communicator: peer_request_handler,
        client_communicator: client_request_handler.clone(),
        election_timer: RandomizedElectionTimer::new(1000, 4000),
        limits: NodeLimits::default(),
        operation_log: MemoryOperationLog::new(cluster_configuration.clone()),
        rsm: MemoryRsm::new(),
        state_saver: MockNodeStateSaver::default()
    };

    let node_worker = raft::start_node(node_config
                                       );

    let thread = node_worker.join_handle.join();
    if thread.is_err() {
        panic!("worker panicked!")
    }
}

fn get_communication_timeout() -> Duration{
    Duration::from_millis(500)
}


fn get_client_requests_address(node_id : u64) -> String{
    format!("127.0.0.1:{}", 50000 + node_id)
}

fn get_network_peer_communicator(node_id: u64) -> NetworkPeerCommunicator<StaticTableServiceDiscovery>{
    let table : HashMap<u64, String>  = [
        (1, get_peer_requests_address(1)),
        (2, get_peer_requests_address(2)),
        (3, get_peer_requests_address(3))]
        .iter().cloned().collect();

    let service_discovery = StaticTableServiceDiscovery::new(table);

    let host = get_peer_requests_address(node_id);

    NetworkPeerCommunicator::new(host, node_id,get_communication_timeout(), true, service_discovery)
}


pub fn get_peer_requests_address(node_id : u64) -> String{
    format!("127.0.0.1:{}", 60000 + node_id)
}

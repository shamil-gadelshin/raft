use raft::{NodeWorker, PeerRequestChannels, PeerRequestHandler};
use raft_modules::{
    ClusterConfiguration, MemoryOperationLog, MemoryRsm, NetworkClientCommunicator,
    RandomizedElectionTimer,
};

use crate::create_node_configuration_with_client_handler;

pub fn create_node_with_network<Pc: PeerRequestHandler + PeerRequestChannels>(
    node_id: u64,
    all_nodes: Vec<u64>,
    _: Pc,
) -> (NodeWorker, NetworkClientCommunicator) {
    let peer_communicator =
        super::network_peer_communicator::get_network_peer_communicator(node_id);

    let client_request_handler = NetworkClientCommunicator::new(
        get_client_requests_address(node_id),
        node_id,
        crate::steps::get_client_communication_timeout(),
        true,
    );

    let cluster_config = ClusterConfiguration::new(all_nodes);
    let operation_log = MemoryOperationLog::new(cluster_config.clone());

    let (client_request_handler, node_config) = create_node_configuration_with_client_handler!(
        node_id,
        client_request_handler,
        cluster_config,
        peer_communicator,
        RandomizedElectionTimer::new(2000, 4000),
        MemoryRsm::new(),
        operation_log
    );

    let node_worker = raft::start_node(node_config);

    (node_worker, client_request_handler)
}

fn get_client_requests_address(node_id: u64) -> String {
    format!("127.0.0.1:{}", 50000 + node_id)
}

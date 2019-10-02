use raft::{PeerRequestHandler, NodeWorker, PeerRequestChannels, NodeConfiguration, NodeState, NodeLimits};
use raft_modules::{NetworkClientCommunicator, ClusterConfiguration, MemoryOperationLog, MockNodeStateSaver, RandomizedElectionTimer, MemoryRsm};

pub fn create_node_with_network<Pc: PeerRequestHandler + PeerRequestChannels>(node_id: u64, all_nodes : Vec<u64>, _ : Pc) -> (NodeWorker, NetworkClientCommunicator)
{
	let peer_communicator = super::network_peer_communicator::get_network_peer_communicator(node_id);
	let cluster_config =ClusterConfiguration::new(all_nodes);
	let client_request_handler = NetworkClientCommunicator::new(get_client_requests_address(node_id), node_id, crate::steps::get_client_communication_timeout(), true);
	let operation_log = MemoryOperationLog::new(cluster_config.clone());

	let node_config = NodeConfiguration {
		node_state: NodeState {
			node_id,
			current_term: 0,
			vote_for_id: None
		},
		cluster_configuration: cluster_config.clone(),
		peer_communicator,
		client_communicator: client_request_handler.clone(),
		election_timer: RandomizedElectionTimer::new(2000, 4000),
		operation_log,
		rsm: MemoryRsm::new(),
		state_saver: MockNodeStateSaver::default(),
		limits: NodeLimits::default()
	};


	let node_worker = raft::start_node(node_config);

	(node_worker, client_request_handler)
}


fn get_client_requests_address(node_id : u64) -> String{
	format!("127.0.0.1:{}", 50000 + node_id)
}


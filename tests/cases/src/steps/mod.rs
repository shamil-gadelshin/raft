use raft_modules::{NetworkClientCommunicator, InProcPeerCommunicator, ClusterConfiguration, MemoryOperationLog, RandomizedElectionTimer, MemoryRsm, MockNodeStateSaver, InProcClientCommunicator};
use std::time::Duration;
use raft::{NodeConfiguration, NodeState, NodeTimings, NodeWorker};
use std::thread;

pub mod cluster;
pub mod peer_communicator;
pub mod data;

pub fn sleep(seconds : u64) {
	thread::sleep(Duration::from_secs(seconds));
}

pub fn get_communication_timeout() -> Duration {
	Duration::from_millis(500)
}

pub fn create_node_with_network(node_id: u64, all_nodes : Vec<u64>, peer_communicator : InProcPeerCommunicator) -> (NodeWorker, NetworkClientCommunicator) {
	let (client_request_handler, node_config) = create_node_configuration_with_network(node_id, all_nodes,peer_communicator );

	let node_worker = raft::start_node(node_config);

	(node_worker, client_request_handler)
}

pub fn create_node_inproc(node_id: u64, all_nodes : Vec<u64>, peer_communicator : InProcPeerCommunicator) -> (NodeWorker, InProcClientCommunicator) {
	let (client_request_handler, node_config) = create_node_configuration_inproc(node_id, all_nodes,peer_communicator );

	let node_worker = raft::start_node(node_config);

	(node_worker, client_request_handler)
}

fn create_node_configuration_with_network(node_id: u64, all_nodes: Vec<u64>, communicator: InProcPeerCommunicator, )
											  -> (NetworkClientCommunicator, NodeConfiguration<MemoryOperationLog, MemoryRsm, NetworkClientCommunicator, InProcPeerCommunicator, RandomizedElectionTimer, MockNodeStateSaver, ClusterConfiguration>)
{
	let cluster_config =ClusterConfiguration::new(all_nodes);
	let client_request_handler = NetworkClientCommunicator::new(get_address(node_id), node_id, get_communication_timeout(), true);
	let operation_log = MemoryOperationLog::new(cluster_config.clone());
	let config = NodeConfiguration {
		node_state: NodeState {
			node_id,
			current_term: 0,
			vote_for_id: None
		},
		cluster_configuration: cluster_config.clone(),
		peer_communicator: communicator,
		client_communicator: client_request_handler.clone(),
		election_timer: RandomizedElectionTimer::new(1000, 4000),
		operation_log,
		rsm: MemoryRsm::default(),
		state_saver: MockNodeStateSaver::default(),
		timings: NodeTimings::default()
	};

	(client_request_handler, config)
}

fn create_node_configuration_inproc(node_id: u64, all_nodes: Vec<u64>, communicator: InProcPeerCommunicator, )
											  -> (InProcClientCommunicator, NodeConfiguration<MemoryOperationLog, MemoryRsm, InProcClientCommunicator, InProcPeerCommunicator, RandomizedElectionTimer, MockNodeStateSaver, ClusterConfiguration>)
{
	let cluster_config =ClusterConfiguration::new(all_nodes);
	let client_request_handler = InProcClientCommunicator::new(node_id, get_communication_timeout());
	let operation_log = MemoryOperationLog::new(cluster_config.clone());
	let config = NodeConfiguration {
		node_state: NodeState {
			node_id,
			current_term: 0,
			vote_for_id: None
		},
		cluster_configuration: cluster_config.clone(),
		peer_communicator: communicator,
		client_communicator: client_request_handler.clone(),
		election_timer: RandomizedElectionTimer::new(1000, 4000),
		operation_log,
		rsm: MemoryRsm::default(),
		state_saver: MockNodeStateSaver::default(),
		timings: NodeTimings::default()
	};

	(client_request_handler, config)
}


fn get_address(node_id : u64) -> String{
	format!("127.0.0.1:{}", 50000 + node_id)
}

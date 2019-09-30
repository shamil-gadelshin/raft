use raft_modules::{NetworkClientCommunicator, ClusterConfiguration, MemoryOperationLog, RandomizedElectionTimer, MemoryRsm, MockNodeStateSaver, InProcClientCommunicator, FixedElectionTimer};
use std::time::Duration;
use raft::{NodeConfiguration, NodeState, NodeTimings, NodeWorker, PeerRequestHandler, PeerRequestChannels, ElectionTimer};
use std::thread;

pub mod cluster;
pub mod peer_communicator;
pub mod data;

pub fn sleep(seconds : u64) {
	thread::sleep(Duration::from_secs(seconds));
}

pub fn get_peers_communication_timeout() -> Duration {
	Duration::from_millis(500)
}

pub fn get_client_communication_timeout() -> Duration {
	Duration::from_millis(2500)
}

pub fn create_node_with_network<Pc>(node_id: u64, all_nodes : Vec<u64>, peer_communicator : Pc) -> (NodeWorker, NetworkClientCommunicator)
where Pc : PeerRequestHandler + PeerRequestChannels{
	let (client_request_handler, node_config) = create_node_configuration_with_network(node_id, all_nodes,peer_communicator );

	let node_worker = raft::start_node(node_config);

	(node_worker, client_request_handler)
}

pub fn create_node_inproc<Pc>(node_id: u64, all_nodes : Vec<u64>, peer_communicator : Pc) -> (NodeWorker, InProcClientCommunicator)
	where  Pc : PeerRequestHandler + PeerRequestChannels {

	if node_id == 1 {
		let election_timer =FixedElectionTimer::new(1000); // leader
		let (client_request_handler, node_config) = create_node_configuration_inproc(node_id, all_nodes, peer_communicator, election_timer);
		let node_worker = raft::start_node(node_config);

		return (node_worker, client_request_handler)
	}

	let election_timer = RandomizedElectionTimer::new(2000, 4000);
	let (client_request_handler, node_config) = create_node_configuration_inproc(node_id, all_nodes, peer_communicator, election_timer);
	let node_worker = raft::start_node(node_config);

	(node_worker, client_request_handler)
}

fn create_node_configuration_with_network<Pc>(node_id: u64, all_nodes: Vec<u64>, communicator: Pc, )
											  -> (NetworkClientCommunicator, NodeConfiguration<MemoryOperationLog, MemoryRsm, NetworkClientCommunicator, Pc, RandomizedElectionTimer, MockNodeStateSaver, ClusterConfiguration>)
	where  Pc : PeerRequestHandler + PeerRequestChannels {
	let cluster_config =ClusterConfiguration::new(all_nodes);
	let client_request_handler = NetworkClientCommunicator::new(get_address(node_id), node_id, get_client_communication_timeout(), true);
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
		election_timer: RandomizedElectionTimer::new(2000, 4000),
		operation_log,
		rsm: MemoryRsm::default(),
		state_saver: MockNodeStateSaver::default(),
		timings: NodeTimings::default()
	};

	(client_request_handler, config)
}

fn create_node_configuration_inproc<Pc, Et>(node_id: u64, all_nodes: Vec<u64>, communicator: Pc, election_timer : Et )
											  -> (InProcClientCommunicator, NodeConfiguration<MemoryOperationLog, MemoryRsm, InProcClientCommunicator, Pc, Et, MockNodeStateSaver, ClusterConfiguration>)
	where  Pc : PeerRequestHandler + PeerRequestChannels,
		   Et : ElectionTimer{
	let cluster_config =ClusterConfiguration::new(all_nodes);
	let client_request_handler = InProcClientCommunicator::new(node_id, get_client_communication_timeout());
	let operation_log = MemoryOperationLog::new(cluster_config.clone());



	let config = NodeConfiguration {
		node_state: NodeState {
			node_id,
			current_term: 3,
			vote_for_id: None
		},
		cluster_configuration: cluster_config.clone(),
		peer_communicator: communicator,
		client_communicator: client_request_handler.clone(),
		election_timer,
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

use raft::{PeerRequestHandler, PeerRequestChannels, NodeTimings, NodeConfiguration, NodeState, ElectionTimer, OperationLog, NodeWorker};
use raft_modules::{ClusterConfiguration, MockNodeStateSaver, MemoryRsm, InProcClientCommunicator, MemoryOperationLog, FixedElectionTimer};
use crate::steps::get_client_communication_timeout;



pub fn create_different_term_node_configuration_inproc<Pc>(node_id: u64, all_nodes: Vec<u64>, communicator: Pc)
													->(NodeWorker, InProcClientCommunicator)
	where  Pc : PeerRequestHandler + PeerRequestChannels{
	let cluster_config =ClusterConfiguration::new(all_nodes);
	let client_request_handler = InProcClientCommunicator::new(node_id, crate::steps::get_client_communication_timeout());
	let operation_log = MemoryOperationLog::new(cluster_config.clone());



	let node_config = NodeConfiguration {
		node_state: NodeState {
			node_id,
			current_term: 3,
			vote_for_id: None
		},
		cluster_configuration: cluster_config.clone(),
		peer_communicator: communicator,
		client_communicator: client_request_handler.clone(),
		election_timer: FixedElectionTimer::new(1000 + node_id * 500),
		operation_log,
		rsm: MemoryRsm::new(),
		state_saver: MockNodeStateSaver::default(),
		timings: NodeTimings::default()
	};

	let node_worker = raft::start_node(node_config);

	(node_worker, client_request_handler)
}



pub fn create_node_configuration_inproc<Pc, Et, Log>(node_id: u64, all_nodes: Vec<u64>, communicator: Pc, election_timer : Et, operation_log: Log )
													 -> (InProcClientCommunicator, NodeConfiguration<Log, MemoryRsm, InProcClientCommunicator, Pc, Et, MockNodeStateSaver, ClusterConfiguration>)
	where  Pc : PeerRequestHandler + PeerRequestChannels,
		   Et : ElectionTimer,
		   Log : OperationLog{
	let cluster_config =ClusterConfiguration::new(all_nodes);
	let client_request_handler = InProcClientCommunicator::new(node_id, get_client_communication_timeout());

	let config = NodeConfiguration {
		node_state: NodeState {
			node_id,
			current_term: 0,
			vote_for_id: None
		},
		cluster_configuration: cluster_config.clone(),
		peer_communicator: communicator,
		client_communicator: client_request_handler.clone(),
		election_timer,
		operation_log,
		rsm: MemoryRsm::new(),
		state_saver: MockNodeStateSaver::default(),
		timings: NodeTimings::default()
	};

	(client_request_handler, config)
}


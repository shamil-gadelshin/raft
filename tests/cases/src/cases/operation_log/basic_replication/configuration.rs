use raft::{PeerRequestHandler, PeerRequestChannels, NodeConfiguration, NodeState, ElectionTimer, OperationLog, NodeLimits};
use raft_modules::{ ClusterConfiguration, MockNodeStateSaver, MemoryRsm, InProcClientCommunicator};
use crate::steps::get_client_communication_timeout;


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
		limits: NodeLimits::default()
	};

	(client_request_handler, config)
}

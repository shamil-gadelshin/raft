use raft::{PeerRequestHandler, PeerRequestChannels, NodeTimings, NodeConfiguration, NodeState, ElectionTimer, ReplicatedStateMachine};
use raft_modules::{ClusterConfiguration, MockNodeStateSaver, InProcClientCommunicator, MemoryOperationLog};
use crate::steps::get_client_communication_timeout;


pub fn create_node_configuration_inproc<Pc, Et, Rsm>(node_id: u64, all_nodes: Vec<u64>, communicator: Pc, election_timer : Et, rsm: Rsm )
												 -> (InProcClientCommunicator, NodeConfiguration<MemoryOperationLog, Rsm, InProcClientCommunicator, Pc, Et, MockNodeStateSaver, ClusterConfiguration>)
	where  Pc : PeerRequestHandler + PeerRequestChannels,
		   Et : ElectionTimer,
		   Rsm : ReplicatedStateMachine{
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
		operation_log: MemoryOperationLog::new(cluster_config.clone()),
		rsm,
		state_saver: MockNodeStateSaver::default(),
		timings: NodeTimings::default()
	};

	(client_request_handler, config)
}

use raft::{PeerRequestHandler, PeerRequestChannels, NodeConfiguration, NodeState, NodeLimits, NodeWorker};
use raft_modules::{ClusterConfiguration, MockNodeStateSaver, MemoryRsm, InProcClientCommunicator, FixedElectionTimer, MemoryOperationLog};


pub fn create_custom_node_inproc<Pc>(node_id: u64, all_nodes : Vec<u64>, peer_communicator : Pc) -> (NodeWorker, InProcClientCommunicator)
	where  Pc : PeerRequestHandler + PeerRequestChannels {
	let election_timer = FixedElectionTimer::new(1000); // leader

	let cluster_config =ClusterConfiguration::new(all_nodes);
	let client_request_handler = InProcClientCommunicator::new(node_id, crate::steps::get_client_communication_timeout());
	let operation_log = MemoryOperationLog::new(cluster_config.clone());

	let mut node_limits = NodeLimits::default();
	node_limits.max_data_content_size = 15;
	let node_config = NodeConfiguration {
		node_state: NodeState {
			node_id,
			current_term: 0,
			vote_for_id: None
		},
		cluster_configuration: cluster_config.clone(),
		peer_communicator,
		client_communicator: client_request_handler.clone(),
		election_timer,
		operation_log,
		rsm: MemoryRsm::new(),
		state_saver: MockNodeStateSaver::default(),
		limits: node_limits
	};
	let node_worker = raft::start_node(node_config);

	return (node_worker, client_request_handler)
}

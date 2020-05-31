#[macro_export]
macro_rules! create_node_configuration {
    // Generic
    (
        $node_id:expr,
        $client_request_handler:expr,
        $cluster_config:expr,
        $communicator:expr,
        $election_timer:expr,
        $rsm:expr,
        $operation_log:expr
    ) => {{
        let config = raft::NodeConfiguration {
            node_state: raft::NodeState {
                node_id: $node_id,
                current_term: 0,
                vote_for_id: None,
            },
            cluster_configuration: $cluster_config,
            peer_communicator: $communicator,
            client_communicator: $client_request_handler.clone(),
            election_timer: $election_timer,
            operation_log: $operation_log,
            rsm: $rsm,
            state_saver: raft_modules::MockNodeStateSaver::default(),
            limits: raft::NodeLimits::default(),
        };

        ($client_request_handler, config)
    }};
    // Generate log and cluster config
    (
        $node_id:expr,
        $all_nodes:expr,
        $communicator:expr,
        $election_timer:expr,
        $rsm:expr
    ) => {{
        let cluster_config = raft_modules::ClusterConfiguration::new($all_nodes);
        let operation_log = raft_modules::MemoryOperationLog::new(cluster_config.clone());

        crate::create_node_configuration!(
            $node_id,
            cluster_config,
            $communicator,
            $election_timer,
            $rsm,
            operation_log
        )
    }};
    // Generate InProcClientCommunicator
    (
        $node_id:expr,
        $cluster_config:expr,
        $communicator:expr,
        $election_timer:expr,
        $rsm:expr,
        $operation_log:expr
    ) => {{
        let client_request_handler = raft_modules::InProcClientCommunicator::new(
            $node_id,
            crate::steps::get_client_communication_timeout(),
        );

        crate::create_node_configuration!(
            $node_id,
            client_request_handler,
            $cluster_config,
            $communicator,
            $election_timer,
            $rsm,
            $operation_log
        )
    }};
}

#[macro_export]
macro_rules! create_node_worker {
    (
        $($tail:tt)*
    ) => {{
        let (client_request_handler, node_config) = crate::create_node_configuration!($($tail)*);

		let node_worker = raft::start_node(node_config);

		(node_worker, client_request_handler)
    }}
}

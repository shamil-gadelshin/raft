#[macro_export]
macro_rules! create_node_configuration {
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

        let config = raft::NodeConfiguration {
            node_state: raft::NodeState {
                node_id: $node_id,
                current_term: 0,
                vote_for_id: None,
            },
            cluster_configuration: $cluster_config,
            peer_communicator: $communicator,
            client_communicator: client_request_handler.clone(),
            election_timer: $election_timer,
            operation_log: $operation_log,
            rsm: $rsm,
            state_saver: raft_modules::MockNodeStateSaver::default(),
            limits: raft::NodeLimits::default(),
        };

        (client_request_handler, config)
    }};
}

#[macro_export]
macro_rules! create_node_configuration_in_proc {
    (
        $node_id:expr,
        $all_nodes:expr,
        $communicator:expr,
        $election_timer:expr,
        $rsm:expr,
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
}

//
// pub fn create_node_configuration_inproc<Pc, Et, Rsm>(
//     node_id: u64,
//     all_nodes: Vec<u64>,
//     communicator: Pc,
//     election_timer: Et,
//     rsm: Rsm,
// ) -> (
//     InProcClientCommunicator,
//     NodeConfiguration<
//         MemoryOperationLog,
//         Rsm,
//         InProcClientCommunicator,
//         Pc,
//         Et,
//         MockNodeStateSaver,
//         ClusterConfiguration,
//     >,
// )
// where
//     Pc: PeerRequestHandler + PeerRequestChannels,
//     Et: ElectionTimer,
//     Rsm: ReplicatedStateMachine,
// {
//     let cluster_config = ClusterConfiguration::new(all_nodes);
//     let operation_log = MemoryOperationLog::new(cluster_config.clone());
//
//     create_node_configuration!(node_id, cluster_config, communicator, election_timer, rsm, operation_log)
// }

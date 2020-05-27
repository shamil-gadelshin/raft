use raft::{NodeLimits, NodeWorker, PeerRequestChannels, PeerRequestHandler, NodeConfiguration};
use raft_modules::{FixedElectionTimer, InProcClientCommunicator, };

pub fn create_custom_node_inproc<Pc>(
    node_id: u64,
    all_nodes: Vec<u64>,
    peer_communicator: Pc,
) -> (NodeWorker, InProcClientCommunicator)
where
    Pc: PeerRequestHandler + PeerRequestChannels,
{
    let election_timer = FixedElectionTimer::new(1000); // leader
    let mut node_limits = NodeLimits::default();
    node_limits.max_data_content_size = 15;

    let (client_request_handler, generic_node_config)
        = crate::steps::create_generic_node_configuration_inproc(node_id,
                                                                 all_nodes,
                                                                 peer_communicator,
                                                                 election_timer);

    let node_config = NodeConfiguration{
        limits: node_limits,
        ..generic_node_config
    };

    let node_worker = raft::start_node(node_config);

    return (node_worker, client_request_handler);
}

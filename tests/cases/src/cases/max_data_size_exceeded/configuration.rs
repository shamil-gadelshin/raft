use raft::{NodeConfiguration, NodeLimits, NodeWorker, PeerRequestChannels, PeerRequestHandler};
use raft_modules::{FixedElectionTimer, InProcClientCommunicator, MemoryRsm};

use crate::create_node_configuration_in_proc;

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

    let (client_request_handler, generic_node_config) = create_node_configuration_in_proc!(
        node_id,
        all_nodes,
        peer_communicator,
        election_timer,
        MemoryRsm::new()
    );

    let node_config = NodeConfiguration {
        limits: node_limits,
        ..generic_node_config
    };

    let node_worker = raft::start_node(node_config);

    (node_worker, client_request_handler)
}

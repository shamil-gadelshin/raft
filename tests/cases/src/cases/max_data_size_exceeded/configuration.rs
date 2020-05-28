use raft::{NodeConfiguration, NodeLimits, NodeWorker, PeerRequestChannels, PeerRequestHandler};
use raft_modules::{FixedElectionTimer, InProcClientCommunicator, MemoryRsm};

use crate::create_node_configuration;

pub fn create_custom_node_inproc<Pc>(
    node_id: u64,
    all_nodes: Vec<u64>,
    peer_communicator: Pc,
) -> (NodeWorker, InProcClientCommunicator)
where
    Pc: PeerRequestHandler + PeerRequestChannels,
{
    let (client_request_handler, generic_node_config) = create_node_configuration!(
        node_id,
        all_nodes,
        peer_communicator,
        FixedElectionTimer::new(1000), // leader,
        MemoryRsm::new()
    );

    let node_config = NodeConfiguration {
        limits: NodeLimits {
            max_data_content_size: 15,
            ..Default::default()
        },
        ..generic_node_config
    };

    let node_worker = raft::start_node(node_config);

    (node_worker, client_request_handler)
}

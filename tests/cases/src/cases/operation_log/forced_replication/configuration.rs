use raft::{NodeConfiguration, NodeState, NodeWorker, PeerRequestChannels, PeerRequestHandler};
use raft_modules::{FixedElectionTimer, InProcClientCommunicator, MemoryRsm};

use crate::create_node_configuration_in_proc;

pub fn create_different_term_node_configuration_inproc<Pc>(
    node_id: u64,
    all_nodes: Vec<u64>,
    communicator: Pc,
) -> (NodeWorker, InProcClientCommunicator)
where
    Pc: PeerRequestHandler + PeerRequestChannels,
{
    let (client_request_handler, mut node_config) = create_node_configuration_in_proc!(
        node_id,
        all_nodes,
        communicator,
        FixedElectionTimer::new(1000 + node_id * 500),
        MemoryRsm::new()
    );

    node_config = NodeConfiguration {
        node_state: NodeState {
            node_id,
            current_term: 3,
            vote_for_id: None,
        },
        ..node_config
    };

    let node_worker = raft::start_node(node_config);

    (node_worker, client_request_handler)
}

use raft::{NodeWorker, PeerRequestChannels, PeerRequestHandler};
use raft_modules::{
    FixedElectionTimer, InProcClientCommunicator, InProcPeerCommunicator, MemoryRsm,
    RandomizedElectionTimer,
};
use std::thread;
use std::time::Duration;

pub mod cluster;
pub mod configuration;
pub mod data;

use crate::create_node_configuration;

pub fn get_generic_peer_communicator(nodes: Vec<u64>) -> InProcPeerCommunicator {
    InProcPeerCommunicator::new(nodes, get_peers_communication_timeout())
}

pub fn get_peers_communication_timeout() -> Duration {
    Duration::from_millis(500)
}

pub fn get_client_communication_timeout() -> Duration {
    Duration::from_millis(2500)
}

pub fn sleep(seconds: u64) {
    thread::sleep(Duration::from_secs(seconds));
}

pub fn create_generic_node_inproc<Pc>(
    node_id: u64,
    all_nodes: Vec<u64>,
    peer_communicator: Pc,
) -> (NodeWorker, InProcClientCommunicator)
where
    Pc: PeerRequestHandler + PeerRequestChannels,
{
    if node_id == 1 {
        let (client_request_handler, node_config) = create_node_configuration!(
            node_id,
            all_nodes,
            peer_communicator,
            FixedElectionTimer::new(1000), // leader
            MemoryRsm::new()
        );
        let node_worker = raft::start_node(node_config);

        return (node_worker, client_request_handler);
    }

    let (client_request_handler, node_config) = create_node_configuration!(
        node_id,
        all_nodes,
        peer_communicator,
        RandomizedElectionTimer::new(2000, 4000),
        MemoryRsm::new()
    );
    let node_worker = raft::start_node(node_config);

    (node_worker, client_request_handler)
}

use crate::communication::network::peer_communicator::service_discovery::PeerCommunicatorServiceDiscovery;
use std::collections::HashMap;

/// Table based implementation of the PeerCommunicatorServiceDiscovery trait.
/// Returns network address of the node from preconfigured HashMap.
#[derive(Clone, Debug)]
pub struct StaticTableServiceDiscovery {
    table: HashMap<u64, String>,
}

impl StaticTableServiceDiscovery {
    /// Creates StaticTableServiceDiscovery using preconfigured HashMap.
    pub fn new(table: HashMap<u64, String>) -> StaticTableServiceDiscovery {
        StaticTableServiceDiscovery { table }
    }
}

impl PeerCommunicatorServiceDiscovery for StaticTableServiceDiscovery {
    fn get_address(&self, node_id: u64) -> String {
        self.table[&node_id].clone()
    }
}

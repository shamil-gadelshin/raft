pub mod static_table;

use std::fmt::Debug;

pub trait PeerCommunicatorServiceDiscovery: Clone + Sync + Send + Debug + 'static {
    fn get_address(&self, node_id: u64) -> String;
}

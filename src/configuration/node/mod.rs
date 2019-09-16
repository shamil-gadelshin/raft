use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::configuration::cluster::{ClusterConfiguration};
use crate::communication::peers::{PeerRequestChannels};
use crate::communication::client::{ClientRequestChannels};
use crate::state::NodeState;


pub trait ElectionTimer: Send + 'static {
    fn get_next_elections_timeout(&self) -> Duration;
}



pub struct NodeConfiguration<Cc, Pc, Et>
where Cc : ClientRequestChannels,
      Pc : PeerRequestChannels + PeerRequestChannels,
      Et : ElectionTimer {
    pub node_state: NodeState,
    pub cluster_configuration: Arc<Mutex<ClusterConfiguration>>,
    pub peer_communicator: Pc,
    pub client_communicator: Cc,
    pub election_timer: Et
}
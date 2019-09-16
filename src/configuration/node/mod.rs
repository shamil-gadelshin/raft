use std::sync::{Arc, Mutex};


use crate::configuration::cluster::{ClusterConfiguration};
use crate::communication::peers::{PeerRequestChannels};
use crate::communication::client::{ClientRequestChannels};
use crate::state::NodeState;

pub struct NodeConfiguration<Cc : ClientRequestChannels, Pc : PeerRequestChannels + PeerRequestChannels > {
    pub node_state: NodeState,
    pub cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
    pub peer_communicator: Pc,
    pub client_communicator: Cc
}
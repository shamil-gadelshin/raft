use std::sync::{Arc, Mutex};


use crate::configuration::cluster::{ClusterConfiguration};
use crate::communication::peers::{PeerRequestChannels};
use crate::communication::client::{ClientRequestChannels};

pub struct NodeConfiguration<Cc : ClientRequestChannels, Pc : PeerRequestChannels + PeerRequestChannels + Clone> {
    pub node_id: u64,
    pub cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
    pub peer_communicator: Pc,
    pub client_communicator: Cc
}
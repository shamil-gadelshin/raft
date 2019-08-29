use std::sync::{Arc, Mutex};


use crate::configuration::cluster::{ClusterConfiguration};
use crate::communication::peers::{InProcNodeCommunicator};
use crate::communication::client::{InProcClientCommunicator};

pub struct NodeConfiguration {
    pub node_id: u64,
    pub cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
    pub peer_communicator: InProcNodeCommunicator,
    pub client_communicator: InProcClientCommunicator
}
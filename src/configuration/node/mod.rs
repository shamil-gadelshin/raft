use std::sync::{Arc, Mutex};


use crate::configuration::cluster::{ClusterConfiguration};
use crate::communication::peers::{InProcNodeCommunicator};
use crate::communication::client::{ClientRequestHandler};

pub struct NodeConfiguration<Storage : Sized> {
    pub node_id: u64,
    pub cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
    pub peer_communicator: InProcNodeCommunicator,
    pub client_request_handler : ClientRequestHandler,
    pub storage : Storage, //TODO private & new()
}
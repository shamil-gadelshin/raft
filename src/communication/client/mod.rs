use std::sync::Arc;
use std::time::Duration;
use crossbeam_channel::{Sender, Receiver};

use crate::communication::duplex_channel::DuplexChannel;
use std::error::Error;


#[derive(Clone, Copy, Debug)]
pub enum ClientResponseStatus {
    Ok,
    NotLeader
}

#[derive(Clone, Copy, Debug)]
pub struct AddServerRequest {
    pub new_server : u64
}

#[derive(Clone, Copy, Debug)]
pub struct ClientRpcResponse {
    pub status : ClientResponseStatus,
    pub current_leader : Option<u64>
}

#[derive(Clone, Debug)]
pub struct NewDataRequest {
    pub data : Arc<&'static [u8]>
}

#[derive(Clone)]
pub struct InProcClientCommunicator {
    add_server_duplex_channel: DuplexChannel<AddServerRequest, ClientRpcResponse>,
    new_data_duplex_channel: DuplexChannel<NewDataRequest, ClientRpcResponse>
}

impl InProcClientCommunicator {
    pub fn new(node_id : u64, timeout : Duration) -> InProcClientCommunicator {
        InProcClientCommunicator {
            add_server_duplex_channel: DuplexChannel::new(format!("AddServer channel NodeId={}", node_id), timeout),
            new_data_duplex_channel: DuplexChannel::new(format!("NewData channel NodeId={}", node_id), timeout)
        }
    }

    pub fn get_add_server_request_rx(&self) -> Receiver<AddServerRequest> {
        self.add_server_duplex_channel.get_request_rx()
    }

    pub fn get_add_server_response_tx(&self) -> Sender<ClientRpcResponse> {
        self.add_server_duplex_channel.get_response_tx()
    }

    pub fn get_new_data_request_rx(&self) -> Receiver<NewDataRequest> {
        self.new_data_duplex_channel.get_request_rx()
    }

    pub fn get_new_data_response_tx(&self) -> Sender<ClientRpcResponse> {
        self.new_data_duplex_channel.get_response_tx()
    }

    //TODO consider & change result error type
    pub fn add_server(&self, request: AddServerRequest) -> Result<ClientRpcResponse, Box<Error>> {
        trace!("Add server request {:?}", request);
        self.add_server_duplex_channel.send_request(request)
     }

    //TODO consider & change result error type
    pub fn new_data(&self, request: NewDataRequest) -> Result<ClientRpcResponse, Box<Error>> {
        trace!("New data request {:?}", request);
        self.new_data_duplex_channel.send_request(request)
     }
}


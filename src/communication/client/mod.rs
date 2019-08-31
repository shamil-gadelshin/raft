use crossbeam_channel::{Sender, Receiver};

use crate::communication::duplex_channel::DuplexChannel;
use std::sync::Arc;

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
pub struct AddServerResponse {
    pub status : ClientResponseStatus,
    pub current_leader : Option<u64>
}

#[derive(Clone, Debug)]
pub struct NewDataRequest {
    pub data : Arc<&'static [u8]>
}

#[derive(Clone, Copy, Debug)]
pub struct NewDataResponse {
    pub status : ClientResponseStatus,
    pub current_leader : Option<u64>
}


#[derive(Clone)]
pub struct InProcClientCommunicator {
    add_server_duplex_channel: DuplexChannel<AddServerRequest, AddServerResponse>,
    new_data_duplex_channel: DuplexChannel<NewDataRequest, NewDataResponse>
}

impl InProcClientCommunicator {
    pub fn new() -> InProcClientCommunicator {

        let client = InProcClientCommunicator {
            add_server_duplex_channel: DuplexChannel::new(),
            new_data_duplex_channel: DuplexChannel::new()
        };

        client
    }

    pub fn get_add_server_request_rx(&self) -> Receiver<AddServerRequest> {
        self.add_server_duplex_channel.get_request_rx()
    }

    pub fn get_add_server_response_tx(&self) -> Sender<AddServerResponse> {
        self.add_server_duplex_channel.get_response_tx()
    }

    pub fn get_new_data_request_rx(&self) -> Receiver<NewDataRequest> {
        self.new_data_duplex_channel.get_request_rx()
    }

    pub fn get_new_data_response_tx(&self) -> Sender<NewDataResponse> {
        self.new_data_duplex_channel.get_response_tx()
    }

    //TODO consider & change result error type
    pub fn add_server(&self, request: AddServerRequest) -> Result<AddServerResponse, &'static str> {
        trace!("Add server request {:?}", request);
        return self.add_server_duplex_channel.send_request(request);
     }

    //TODO consider & change result error type
    pub fn new_data(&self, request: NewDataRequest) -> Result<NewDataResponse, &'static str> {
        trace!("New data request {:?}", request);
        return self.new_data_duplex_channel.send_request(request);
     }
}


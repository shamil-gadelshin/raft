use crossbeam_channel::{Sender, Receiver};

use std::time::Duration;
use crate::communication::duplex_channel::DuplexChannel;

#[derive(Clone, Copy, Debug)]
pub enum ChangeMembershipResponseStatus {
    Ok,
    NotLeader
}

#[derive(Clone, Copy, Debug)]
pub struct AddServerRequest {
    pub new_server : u64
}

#[derive(Clone, Copy, Debug)]
pub struct AddServerResponse {
    pub status : ChangeMembershipResponseStatus,
    pub current_leader : Option<u64>
}


#[derive(Clone)]
pub struct InProcClientCommunicator {
    add_server_duplex_channel: DuplexChannel<AddServerRequest, AddServerResponse>
}

impl InProcClientCommunicator {
    pub fn new() -> InProcClientCommunicator {

        let client = InProcClientCommunicator {
            add_server_duplex_channel: DuplexChannel::new()
        };

        client
    }

    pub fn get_add_server_request_rx(&self) -> Receiver<AddServerRequest> {
        self.add_server_duplex_channel.get_request_rx()
    }

    pub fn get_add_server_response_tx(&self) -> Sender<AddServerResponse> {
        self.add_server_duplex_channel.get_response_tx()
    }

    //TODO consider & change result error type
    pub fn add_server(&self, request: AddServerRequest) -> Result<AddServerResponse, &'static str> {
        trace!("Add server request {:?}", request);
        return self.add_server_duplex_channel.send_request(request);
     }
}


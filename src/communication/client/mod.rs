use crossbeam_channel::{Sender, Receiver};

use crate::core::*;
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
pub struct ClientRequestHandler {
    duplex_channel : DuplexChannel<AddServerRequest, AddServerResponse>
}

impl ClientRequestHandler {
    pub fn new() -> ClientRequestHandler {

        let client = ClientRequestHandler{
            duplex_channel : DuplexChannel::new()
        };

        client
    }

    pub fn get_add_server_request_rx(&self) -> Receiver<AddServerRequest> {
        self.duplex_channel.get_request_rx()
    }

    pub fn get_add_server_response_tx(&self) -> Sender<AddServerResponse> {
        self.duplex_channel.get_response_tx()
    }

    //TODO change result error type
    pub fn add_server(&self, request: AddServerRequest) -> Result<AddServerResponse, &'static str> {
        print_event( format!("Add server request {:?}", request));

        let timeout = crossbeam_channel::after(Duration::new(1,0));
        select!(
            recv(timeout) -> _  => {
                return Err("Send add_server_request timeout")
 //TODO               return Err(format!("Send add_rpc_timeout - Destination Node {:?} Sending request {:?}",destination_node_id, request))
            },
            send(self.duplex_channel.request_tx, request) -> res => {
                if let Err(err) = res {
                    return Err("Cannot send add_server_request")
                }
            },
        );

        select!(
            recv(timeout) -> _  => {
                return Err("Receive add_server_response timeout")
            },
            recv(self.duplex_channel.response_rx) -> res => {
                if let Err(err) = res {
                    return Err("Cannot receive add_server_response")
                }
                if let Ok(resp) = res {
                    let add_server_response = resp;

                    return Ok(add_server_response);
                }
            },
        );

        panic!("invalid add_server request-response sequence");
    }
}


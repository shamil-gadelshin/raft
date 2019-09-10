use std::sync::Arc;
use crossbeam_channel::{Sender, Receiver};

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

pub trait ClientRequestHandler {
    fn add_server(&self, request: AddServerRequest) -> Result<ClientRpcResponse, Box<Error>>;
    fn new_data(&self, request: NewDataRequest) -> Result<ClientRpcResponse, Box<Error>>;
}

pub trait ClientRequestChannels {
    fn add_server_request_rx(&self) -> Receiver<AddServerRequest>;
    fn add_server_response_tx(&self) -> Sender<ClientRpcResponse>;
    fn new_data_request_rx(&self) -> Receiver<NewDataRequest>;
    fn new_data_response_tx(&self) -> Sender<ClientRpcResponse>;
}

#[derive(Clone, Debug)]
pub struct NewDataRequest {
    pub data : Arc<&'static [u8]>
}


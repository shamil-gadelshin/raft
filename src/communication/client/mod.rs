use std::sync::Arc;

use crossbeam_channel::{Sender, Receiver};
use crate::errors::RaftError;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClientResponseStatus {
    Ok,
    NoQuorum,
    NotLeader,
    Error
}

#[derive(Clone, Copy, Debug)]
pub struct AddServerRequest {
    pub new_server : u64
}

#[derive(Clone, Debug)]
pub struct NewDataRequest {
    pub data : Arc<&'static [u8]>
}

#[derive(Clone, Debug)]
pub struct ClientRpcResponse {
    pub status : ClientResponseStatus,
    pub current_leader : Option<u64>,
    pub message: String
}

pub trait ClientRequestHandler: Clone + Sync + Send + 'static  {
    fn add_server(&self, request: AddServerRequest) -> Result<ClientRpcResponse, RaftError>;
    fn new_data(&self, request: NewDataRequest) -> Result<ClientRpcResponse, RaftError>;
}

pub trait ClientRequestChannels:  Send + Clone+ 'static + {
    fn add_server_request_rx(&self) -> Receiver<AddServerRequest>;
    fn add_server_response_tx(&self) -> Sender<ClientRpcResponse>;
    fn new_data_request_rx(&self) -> Receiver<NewDataRequest>;
    fn new_data_response_tx(&self) -> Sender<ClientRpcResponse>;
}



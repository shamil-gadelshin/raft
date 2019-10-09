use std::sync::Arc;

use crate::errors::RaftError;
use crossbeam_channel::{Receiver, Sender};

/// Status of client RPC response.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Display)]
pub enum ClientResponseStatus {
    /// Successful request.
    Ok,

    /// Leader cannot get the quorum of the followers for this request.
    NoQuorum,

    /// This node is not a leader.
    NotLeader,

    /// Error occurred.
    Error,
}

/// Client RPC request for add new server to the cluster.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Default, Display)]
#[display(fmt = "New server {}", new_server)]
pub struct AddServerRequest {
    /// New node id.
    pub new_server: u64,
}

/// Client RPC request for the apply new data(command) to the operation log.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Display)]
#[display(fmt = "Data size: {}", "data.len()")]
pub struct NewDataRequest {
    /// Data(command) serialized in bytes format.
    pub data: Arc<&'static [u8]>,
}

/// Generic response for the client RPC
#[derive(Clone, Debug, Eq, PartialEq, Hash, Display)]
#[display(fmt = "Client RPC response: status {} current_leader {:?} message: {}",
    status, current_leader, message )]
pub struct ClientRpcResponse {
    /// Response status.
    pub status: ClientResponseStatus,

    /// Current leader hint. Can be empty if no leader known to the moment.
    pub current_leader: Option<u64>,

    /// Error message. Not empty if status is 'Error'.
    pub message: String,
}

/// API abstraction for the communications with clients.
pub trait ClientRequestHandler: Clone + Sync + Send + 'static {
    /// Adds new server to the cluster.
    fn add_server(&self, request: AddServerRequest) -> Result<ClientRpcResponse, RaftError>;

    /// Apply new data(command) to operation log.
    fn new_data(&self, request: NewDataRequest) -> Result<ClientRpcResponse, RaftError>;
}

/// Abstraction for channels responsible for the communications with clients.
pub trait ClientRequestChannels: Send + Clone + 'static {
    /// Returns receiver channel for 'add new server to the cluster' requests.
    fn add_server_request_rx(&self) -> Receiver<AddServerRequest>;

    /// Returns sender channel for 'add new server to the cluster' responses.
    fn add_server_response_tx(&self) -> Sender<ClientRpcResponse>;

    /// Returns receiver channel for apply new data(command) to operation log requests.
    fn new_data_request_rx(&self) -> Receiver<NewDataRequest>;

    /// Returns sender channel for apply new data(command) to operation log responses.
    fn new_data_response_tx(&self) -> Sender<ClientRpcResponse>;
}

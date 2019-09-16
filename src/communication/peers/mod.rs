use crossbeam_channel::{Sender, Receiver};

use crate::common::{LogEntry};
use crate::common::QuorumResponse;
use std::error::Error;

#[derive(Clone, Copy, Debug)]
pub struct VoteRequest {
    pub term : u64,
    pub candidate_id : u64,
    pub last_log_term : u64,
    pub last_log_index : u64,
}

#[derive(Clone, Copy, Debug)]
pub struct VoteResponse {
    pub term : u64,
    pub vote_granted: bool,
    pub peer_id: u64
}

impl QuorumResponse for VoteResponse {
    fn get_result(&self) -> bool {
        self.vote_granted
    }
}

#[derive(Clone, Debug)]
pub struct AppendEntriesRequest {
    pub term : u64,
    pub prev_log_term : u64,
    pub prev_log_index : u64,
    pub leader_id : u64,
    pub leader_commit : u64,
    pub entries : Vec<LogEntry>
}

//TODO add failure reason
#[derive(Clone, Copy, Debug)]
pub struct AppendEntriesResponse {
    pub term : u64,
    pub success : bool
}

impl QuorumResponse for AppendEntriesResponse {
    fn get_result(&self) -> bool {
        self.success
    }
}

pub trait PeerRequestHandler: Sync + Send + 'static + Clone {
    fn send_vote_request(&self, destination_node_id: u64, request: VoteRequest)-> Result<VoteResponse, Box<Error>>;
    fn send_append_entries_request(&self, destination_node_id: u64, request: AppendEntriesRequest) -> Result<AppendEntriesResponse, Box<Error>>;
}

pub trait PeerRequestChannels {
    fn vote_request_rx(&self, node_id : u64) -> Receiver<VoteRequest>;
    fn vote_response_tx(&self, node_id : u64) -> Sender<VoteResponse>;
    fn append_entries_request_rx(&self, node_id : u64) -> Receiver<AppendEntriesRequest>;
    fn append_entries_response_tx(&self, node_id : u64) -> Sender<AppendEntriesResponse>;
}
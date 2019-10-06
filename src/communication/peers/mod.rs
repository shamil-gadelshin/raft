use crossbeam_channel::{Receiver, Sender};

use crate::errors::RaftError;
use crate::operation_log::LogEntry;
use crate::operation_log::QuorumResponse;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Default, Display)]
#[display(fmt = "Term {} Candidate_id {}, Last log: term {} index {}",
    term, candidate_id, last_log_term, last_log_index)]
pub struct VoteRequest {
    pub term: u64,
    pub candidate_id: u64,
    pub last_log_term: u64,
    pub last_log_index: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Default, Display)]
#[display(fmt = "Term {} Peer_id {} Vote_granted - {}", term, peer_id, vote_granted)]
pub struct VoteResponse {
    pub term: u64,
    pub vote_granted: bool,
    pub peer_id: u64,
}

impl QuorumResponse for VoteResponse {
    fn result(&self) -> bool {
        self.vote_granted
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default, Display)]
#[display(fmt = "Term {} Leader_id {}, Prev log: term {} index {}, Leader commit {}. Entries - {}",
    term, leader_id, prev_log_term, prev_log_index, leader_commit, "entries.len()")]
pub struct AppendEntriesRequest {
    pub term: u64,
    pub prev_log_term: u64,
    pub prev_log_index: u64,
    pub leader_id: u64,
    pub leader_commit: u64,
    pub entries: Vec<LogEntry>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Default, Display)]
#[display(fmt = "Term {} Success - {}", term, success)]
pub struct AppendEntriesResponse {
    pub term: u64,
    pub success: bool,
}

impl QuorumResponse for AppendEntriesResponse {
    fn result(&self) -> bool {
        self.success
    }
}

pub trait PeerRequestHandler: Send + Sync + 'static + Clone {
    fn send_vote_request(&self, dest_node_id: u64, request: VoteRequest)
        -> Result<VoteResponse, RaftError>;

    fn send_append_entries_request(&self, dest_node_id: u64, request: AppendEntriesRequest)
        -> Result<AppendEntriesResponse, RaftError>;
}

pub trait PeerRequestChannels {
    fn vote_request_rx(&self, node_id: u64) -> Receiver<VoteRequest>;
    fn vote_response_tx(&self, node_id: u64) -> Sender<VoteResponse>;
    fn append_entries_request_rx(&self, node_id: u64) -> Receiver<AppendEntriesRequest>;
    fn append_entries_response_tx(&self, node_id: u64) -> Sender<AppendEntriesResponse>;
}

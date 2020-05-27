use crossbeam_channel::{Receiver, Sender};

use crate::errors::RaftError;
use crate::operation_log::{LogEntry, QuorumResponse};

/// Leadership election vote request.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Default, Display)]
#[display(
    fmt = "Term {} Candidate_id {}, Last log: term {} index {}",
    term,
    candidate_id,
    last_log_term,
    last_log_index
)]
pub struct VoteRequest {
    /// Current elections term.
    pub term: u64,

    /// Node Id for the elections.
    pub candidate_id: u64,

    /// Last operation log entry term from candidate. Can affect vote decision.
    pub last_log_term: u64,

    /// Last operation log entry index from candidate. Can affect vote decision.
    pub last_log_index: u64,
}

/// The response for the leadership elections vote request.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Default, Display)]
#[display(
    fmt = "Term {} Peer_id {} Vote_granted - {}",
    term,
    peer_id,
    vote_granted
)]
pub struct VoteResponse {
    /// Current elections term.
    pub term: u64,

    /// Vote response result.
    pub vote_granted: bool,

    /// Response origin.
    pub peer_id: u64,
}

impl QuorumResponse for VoteResponse {
    fn result(&self) -> bool {
        self.vote_granted
    }
}

/// Operation log replication request or heartbeat request.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Default, Display)]
#[display(
    fmt = "Term {} Leader_id {}, Prev log: term {} index {}, Leader commit {}. Entries - {}",
    term,
    leader_id,
    prev_log_term,
    prev_log_index,
    leader_commit,
    "entries.len()"
)]
pub struct AppendEntriesRequest {
    /// Leaders current elections term.
    pub term: u64,

    /// The term of the entry previous to this request.
    pub prev_log_term: u64,

    /// The index of the entry previous to this request.
    pub prev_log_index: u64,

    /// Current leader id.
    pub leader_id: u64,

    /// Current leader commit index. Indicates actual operation log commit index for the follower.
    pub leader_commit: u64,

    /// Operation log entries to replicate. Can be empty for heartbeat.
    pub entries: Vec<LogEntry>,
}

/// Operation log replication or heartbeat result.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Default, Display)]
#[display(fmt = "Term {} Success - {}", term, success)]
pub struct AppendEntriesResponse {
    /// The term of the follower. Can convert sender-leader to follower.
    pub term: u64,

    /// Operation log replication or heartbeat result.
    pub success: bool,
}

impl QuorumResponse for AppendEntriesResponse {
    fn result(&self) -> bool {
        self.success
    }
}

/// API abstraction for the communications with peers.
pub trait PeerRequestHandler: Send + Sync + 'static + Clone {
    /// Send vote request to peer and get vote response.
    fn send_vote_request(
        &self,
        dest_node_id: u64,
        request: VoteRequest,
    ) -> Result<VoteResponse, RaftError>;

    /// Send operation log replication request or heartbeat request to peer and get vote response.
    fn send_append_entries_request(
        &self,
        dest_node_id: u64,
        request: AppendEntriesRequest,
    ) -> Result<AppendEntriesResponse, RaftError>;
}

/// Abstraction for channels responsible for the communications with peers.
pub trait PeerRequestChannels {
    /// Returns receiver channel for vote requests.
    fn vote_request_rx(&self, node_id: u64) -> Receiver<VoteRequest>;

    /// Returns sender channel for vote responses.
    fn vote_response_tx(&self, node_id: u64) -> Sender<VoteResponse>;

    /// Returns receiver channel for operation log replication request or heartbeat requests.
    fn append_entries_request_rx(&self, node_id: u64) -> Receiver<AppendEntriesRequest>;

    /// Returns sender channel for operation log replication request or heartbeat responses.
    fn append_entries_response_tx(&self, node_id: u64) -> Sender<AppendEntriesResponse>;
}

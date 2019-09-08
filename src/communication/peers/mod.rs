use std::collections::HashMap;
use std::time::Duration;

use crossbeam_channel::{Sender, Receiver, SendTimeoutError};

use crate::common::{LogEntry};
use crate::communication::duplex_channel::DuplexChannel;
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

#[derive(Clone, Debug)]
pub struct AppendEntriesRequest {
    pub term : u64,
    pub prev_log_term : u64,
    pub prev_log_index : u64,
    pub leader_id : u64,
    pub leader_commit : u64,
    pub entries : Vec<LogEntry>
}

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

#[derive(Clone,Debug)]
pub struct InProcPeerCommunicator {
    timeout: Duration,
    votes_channels: HashMap<u64,DuplexChannel<VoteRequest, VoteResponse>>,
    append_entries_channels: HashMap<u64,DuplexChannel<AppendEntriesRequest, AppendEntriesResponse>>,
}


impl InProcPeerCommunicator {
    pub fn new(nodes : Vec<u64>, timeout : Duration) -> InProcPeerCommunicator {
        let votes_channels = HashMap::new();
        let append_entries_channels = HashMap::new();

        let mut communicator = InProcPeerCommunicator {
            timeout,
            votes_channels,
            append_entries_channels,
        };

        for node_id in nodes {
            communicator.add_node_communication(node_id);
        }

        communicator
    }

    pub fn add_node_communication(&mut self, node_id : u64) {
        let vote_duplex = DuplexChannel::new(format!("Vote channel NodeId={}", node_id), self.timeout);
        let append_entries_duplex = DuplexChannel::new(format!("AppendEntries channel NodeId={}", node_id), self.timeout);

        self.votes_channels.insert(node_id, vote_duplex);
        self.append_entries_channels.insert(node_id, append_entries_duplex);
    }

    pub fn get_vote_request_rx(&self, node_id : u64) -> Receiver<VoteRequest> {
        self.votes_channels[&node_id].get_request_rx()
    }

    pub fn get_vote_response_rx(&self, node_id : u64) -> Receiver<VoteResponse> {
        self.votes_channels[&node_id].get_response_rx()
    }

    pub fn get_append_entries_request_rx(&self, node_id : u64) -> Receiver<AppendEntriesRequest> {
        self.append_entries_channels[&node_id].get_request_rx()
    }
    pub fn get_append_entries_response_tx(&self, node_id : u64) -> Sender<AppendEntriesResponse> {
        self.append_entries_channels[&node_id].get_response_tx()
    }

    //TODO check node_id existence
    pub fn send_vote_request(&self, destination_node_id: u64, request: VoteRequest)-> Result<(), SendTimeoutError<VoteRequest>> {
        trace!("Destination Node {:?} Sending request {:?}",destination_node_id, request);
        self.votes_channels[&destination_node_id].request_tx.send_timeout(request, self.timeout)
    }
    pub fn send_vote_response(&self, destination_node_id: u64, response: VoteResponse) -> Result<(), SendTimeoutError<VoteResponse>>{
        trace!("Destination Node {:?} Sending response {:?}", destination_node_id, response);
        self.votes_channels[&destination_node_id].response_tx.send_timeout(response, self.timeout)
    }
    pub fn send_append_entries_request(&self, destination_node_id: u64, request: AppendEntriesRequest) -> Result<AppendEntriesResponse, Box<Error>>  {
        trace!("Destination Node {:?} Sending request {:?}",destination_node_id, request);
        self.append_entries_channels[&destination_node_id].send_request(request)
    }
}


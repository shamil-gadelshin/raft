use crossbeam_channel::{Sender};
use std::collections::HashMap;

use super::core::print_event;

#[derive(Clone, Copy, Debug)]
pub struct VoteRequest {
    pub term : u64,
    pub candidate_id : u64
}

#[derive(Clone, Copy, Debug)]
pub struct AppendEntriesRequest {
    pub term : u64,
    pub leader_id : u64
}

#[derive(Clone, Copy, Debug)]
pub struct VoteResponse {
    pub term : u64,
    pub vote_granted: bool,
    pub peer_id: u64
}

#[derive(Clone)]
pub struct InProcNodeCommunicator {
    pub vote_request_channels_tx: HashMap<u64, Sender<VoteRequest>>,
    pub vote_response_channels_tx: HashMap<u64, Sender<VoteResponse>>,
    pub append_entries_request_channels_tx: HashMap<u64, Sender<AppendEntriesRequest>>,
}

impl InProcNodeCommunicator {
    pub fn send_vote_request(&self, destination_node_id: u64, request: VoteRequest) {
        print_event( format!("Destination node {:?} Sending request {:?}",destination_node_id, request));
        self.vote_request_channels_tx[&destination_node_id].send(request).expect("cannot send vote request");
    }
    pub fn send_vote_response(&self, destination_node_id: u64, response: VoteResponse) {
        print_event( format!("Destination node {:?} Sending response {:?}", destination_node_id, response));
        self.vote_response_channels_tx[&destination_node_id].send(response).expect("cannot send vote response");
    }
    pub fn send_append_entries_request(&self, destination_node_id: u64, request: AppendEntriesRequest) {
        print_event( format!("Destination node {:?} Sending request {:?}",destination_node_id, request));
        self.append_entries_request_channels_tx[&destination_node_id].send(request).expect("cannot send vote request");
    }
}


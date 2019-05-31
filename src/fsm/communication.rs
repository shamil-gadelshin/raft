use crossbeam_channel::{Sender};
use std::collections::HashMap;


#[derive(Clone, Copy, Debug)]
pub struct VoteRequest {
    pub term : u64,
    pub candidate_id : u64
}

#[derive(Clone, Copy, Debug)]
pub struct VoteResponse {
    pub term : u64,
    pub voted_for_candidate_id : u64,
    pub peer_id: u64
}

#[derive(Clone)]
pub struct InProcNodeCommunicator {
    pub request_channels_tx: HashMap<u64, Sender<VoteRequest>>,
    pub response_channels_tx: HashMap<u64, Sender<VoteResponse>>,
}

impl InProcNodeCommunicator {
    pub fn send_vote_request(&self, destination_node_id: u64, request: VoteRequest) {
        self.request_channels_tx[&destination_node_id].send(request);
    }
    pub fn send_vote_response(&self, destination_node_id: u64, response: VoteResponse) {
        self.response_channels_tx[&destination_node_id].send(response);
    }
}


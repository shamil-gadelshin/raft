use crossbeam_channel::{Sender, Receiver};
use std::collections::HashMap;

use crate::core::*;

#[derive(Clone, Copy, Debug)]
pub struct VoteRequest {
    pub term : u64,
    pub candidate_id : u64
}

#[derive(Clone, Copy, Debug)]
pub struct VoteResponse {
    pub term : u64,
    pub vote_granted: bool,
    pub peer_id: u64
}

#[derive(Clone, Copy, Debug)]
pub struct AppendEntriesRequest {
    pub term : u64,
    pub leader_id : u64
}

//TODO communicator timeout handling
#[derive(Clone)]
pub struct InProcNodeCommunicator {
    vote_request_channels_tx: HashMap<u64, Sender<VoteRequest>>,
    vote_request_channels_rx: HashMap<u64, Receiver<VoteRequest>>,
    vote_response_channels_tx: HashMap<u64, Sender<VoteResponse>>,
    vote_response_channels_rx: HashMap<u64, Receiver<VoteResponse>>,
    append_entries_request_channels_tx: HashMap<u64, Sender<AppendEntriesRequest>>,
    append_entries_request_channels_rx: HashMap<u64, Receiver<AppendEntriesRequest>>,
}

//TODO extract communicator trait
impl InProcNodeCommunicator {
    pub fn new(nodes : Vec<u64>) -> InProcNodeCommunicator {
        let vote_request_channels_tx = HashMap::new();
        let vote_request_channels_rx = HashMap::new();
        let vote_response_channels_tx = HashMap::new();
        let vote_response_channels_rx = HashMap::new();
        let append_entries_request_channels_tx = HashMap::new();
        let append_entries_request_channels_rx = HashMap::new();

        let mut communicator = InProcNodeCommunicator{
            vote_request_channels_tx,
            vote_request_channels_rx,
            vote_response_channels_tx,
            vote_response_channels_rx,
            append_entries_request_channels_tx,
            append_entries_request_channels_rx,
        };

        for node_id in nodes {
            communicator.add_node_communication(node_id);
        }

        communicator
    }

    pub fn add_node_communication(&mut self, node_id : u64) {
        let (vote_request_tx, vote_request_rx): (Sender<VoteRequest>, Receiver<VoteRequest>) = crossbeam_channel::unbounded();
        let (vote_response_tx, vote_response_rx): (Sender<VoteResponse>, Receiver<VoteResponse>) = crossbeam_channel::unbounded();
        let (append_entries_request_tx, append_entries_request_rx): (Sender<AppendEntriesRequest>, Receiver<AppendEntriesRequest>) = crossbeam_channel::unbounded();

        self.vote_request_channels_tx.insert(node_id, vote_request_tx);
        self.vote_request_channels_rx.insert(node_id, vote_request_rx);
        self.vote_response_channels_tx.insert(node_id, vote_response_tx);
        self.vote_response_channels_rx.insert(node_id, vote_response_rx);
        self.append_entries_request_channels_tx.insert(node_id, append_entries_request_tx);
        self.append_entries_request_channels_rx.insert(node_id, append_entries_request_rx);
    }

    pub fn get_vote_request_channel_rx(&self, node_id : u64) -> Receiver<VoteRequest> {
        self.vote_request_channels_rx[&node_id].clone()
    }

    pub fn get_vote_response_channel_rx(&self, node_id : u64) -> Receiver<VoteResponse> {
        self.vote_response_channels_rx[&node_id].clone()
    }

    pub fn get_append_entries_request_rx(&self, node_id : u64) -> Receiver<AppendEntriesRequest> {
        self.append_entries_request_channels_rx[&node_id].clone()
    }

    //// *** TODO: split creation & function
    //TODO check node_id existence
    //TODO communicator timeout handling
    pub fn send_vote_request(&self, destination_node_id: u64, request: VoteRequest) {
        print_event( format!("Destination Node {:?} Sending request {:?}",destination_node_id, request));
        self.vote_request_channels_tx[&destination_node_id].send(request).expect("cannot send vote request");
    }
    pub fn send_vote_response(&self, destination_node_id: u64, response: VoteResponse) {
        print_event( format!("Destination Node {:?} Sending response {:?}", destination_node_id, response));
        self.vote_response_channels_tx[&destination_node_id].send(response).expect("cannot send vote response");
    }
    pub fn send_append_entries_request(&self, destination_node_id: u64, request: AppendEntriesRequest) {
        print_event( format!("Destination Node {:?} Sending request {:?}",destination_node_id, request));
        self.append_entries_request_channels_tx[&destination_node_id].send(request).expect("cannot send vote request");
    }
}


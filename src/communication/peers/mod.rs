use crossbeam_channel::{Sender, Receiver};
use std::collections::HashMap;

use crate::core::*;
use crate::communication::duplex_channel::DuplexChannel;

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

#[derive(Clone, Copy, Debug)]
pub struct AppendEntriesResponse {
    pub term : u64,
    pub success : bool
}

//TODO communicator timeout handling
#[derive(Clone)]
pub struct InProcNodeCommunicator {
    votes_channels: HashMap<u64,DuplexChannel<VoteRequest, VoteResponse>>,
    append_entries_request_channels_tx: HashMap<u64, Sender<AppendEntriesRequest>>,
    append_entries_request_channels_rx: HashMap<u64, Receiver<AppendEntriesRequest>>,
}

//TODO extract communicator trait
impl InProcNodeCommunicator {
    pub fn new(nodes : Vec<u64>) -> InProcNodeCommunicator {
        let votes_channels = HashMap::new();
        let append_entries_request_channels_tx = HashMap::new();
        let append_entries_request_channels_rx = HashMap::new();

        let mut communicator = InProcNodeCommunicator{
            votes_channels,
            append_entries_request_channels_tx,
            append_entries_request_channels_rx,
        };

        for node_id in nodes {
            communicator.add_node_communication(node_id);
        }

        communicator
    }

    pub fn add_node_communication(&mut self, node_id : u64) {
        let vote_duplex = DuplexChannel::new();
        let (append_entries_request_tx, append_entries_request_rx): (Sender<AppendEntriesRequest>, Receiver<AppendEntriesRequest>) = crossbeam_channel::unbounded();

        self.votes_channels.insert(node_id, vote_duplex);
        self.append_entries_request_channels_tx.insert(node_id, append_entries_request_tx);
        self.append_entries_request_channels_rx.insert(node_id, append_entries_request_rx);
    }

    pub fn get_vote_request_channel_rx(&self, node_id : u64) -> Receiver<VoteRequest> {
        self.votes_channels[&node_id].get_request_rx()
    }

    pub fn get_vote_response_channel_rx(&self, node_id : u64) -> Receiver<VoteResponse> {
        self.votes_channels[&node_id].get_response_rx()
    }

    pub fn get_append_entries_request_rx(&self, node_id : u64) -> Receiver<AppendEntriesRequest> {
        self.append_entries_request_channels_rx[&node_id].clone()
    }

    //// *** TODO: split creation & function
    //TODO check node_id existence
    //TODO communicator timeout handling
    pub fn send_vote_request(&self, destination_node_id: u64, request: VoteRequest) {
        print_event( format!("Destination Node {:?} Sending request {:?}",destination_node_id, request));
        self.votes_channels[&destination_node_id].request_tx.send(request).expect("cannot send vote request");
    }
    pub fn send_vote_response(&self, destination_node_id: u64, response: VoteResponse) {
        print_event( format!("Destination Node {:?} Sending response {:?}", destination_node_id, response));
        self.votes_channels[&destination_node_id].response_tx.send(response).expect("cannot send vote response");
    }
    pub fn send_append_entries_request(&self, destination_node_id: u64, request: AppendEntriesRequest) {
        print_event( format!("Destination Node {:?} Sending request {:?}",destination_node_id, request));
        self.append_entries_request_channels_tx[&destination_node_id].send(request).expect("cannot send vote request");
    }
}


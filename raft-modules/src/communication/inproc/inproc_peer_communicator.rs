use crate::communication::duplex_channel::DuplexChannel;

use crossbeam_channel::{Receiver, Sender};
use raft::RaftError;
use raft::{
    AppendEntriesRequest, AppendEntriesResponse, PeerRequestChannels, PeerRequestHandler,
    VoteRequest, VoteResponse,
};

use std::collections::HashMap;
use std::time::Duration;

/// Basic in-memory implementation of the PeerRequestHandler and  PeerRequestChannels traits.
#[derive(Clone, Debug)]
pub struct InProcPeerCommunicator {
    timeout: Duration,
    votes_channels: HashMap<u64, DuplexChannel<VoteRequest, VoteResponse>>,
    append_entries_channels:
        HashMap<u64, DuplexChannel<AppendEntriesRequest, AppendEntriesResponse>>,
}

impl InProcPeerCommunicator {
    /// Create new instance of the InProcPeerCommunicator with node_id and communication timeout.
    pub fn new(nodes: Vec<u64>, timeout: Duration) -> InProcPeerCommunicator {
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

    fn add_node_communication(&mut self, node_id: u64) {
        let vote_duplex =
            DuplexChannel::new(format!("Vote channel NodeId={}", node_id), self.timeout);
        let append_entries_duplex = DuplexChannel::new(
            format!("AppendEntries channel NodeId={}", node_id),
            self.timeout,
        );

        self.votes_channels.insert(node_id, vote_duplex);
        self.append_entries_channels
            .insert(node_id, append_entries_duplex);
    }
}

impl PeerRequestHandler for InProcPeerCommunicator {
    fn send_vote_request(
        &self,
        destination_node_id: u64,
        request: VoteRequest,
    ) -> Result<VoteResponse, RaftError> {
        trace!(
            "Destination Node {} Sending request {}",
            destination_node_id,
            request
        );

        let resp = self.votes_channels[&destination_node_id].send_request(request);

        trace!(
            "Destination Node {} Response {:?}",
            destination_node_id,
            resp
        );

        resp
    }
    fn send_append_entries_request(
        &self,
        destination_node_id: u64,
        request: AppendEntriesRequest,
    ) -> Result<AppendEntriesResponse, RaftError> {
        trace!(
            "Destination Node {} Sending request {}",
            destination_node_id,
            request
        );

        let resp = self.append_entries_channels[&destination_node_id].send_request(request);

        trace!(
            "Destination Node {} Response {:?}",
            destination_node_id,
            resp
        );

        resp
    }
}

impl PeerRequestChannels for InProcPeerCommunicator {
    fn vote_request_rx(&self, node_id: u64) -> Receiver<VoteRequest> {
        self.votes_channels[&node_id].request_rx()
    }

    fn vote_response_tx(&self, node_id: u64) -> Sender<VoteResponse> {
        self.votes_channels[&node_id].response_tx()
    }

    fn append_entries_request_rx(&self, node_id: u64) -> Receiver<AppendEntriesRequest> {
        self.append_entries_channels[&node_id].request_rx()
    }

    fn append_entries_response_tx(&self, node_id: u64) -> Sender<AppendEntriesResponse> {
        self.append_entries_channels[&node_id].response_tx()
    }
}

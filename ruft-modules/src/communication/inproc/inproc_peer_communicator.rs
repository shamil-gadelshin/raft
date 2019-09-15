use crate::communication::duplex_channel::DuplexChannel;

use crossbeam_channel::{Receiver, Sender};
use ruft::{VoteRequest, VoteResponse, AppendEntriesRequest, AppendEntriesResponse, PeerRequestHandler, PeerRequestChannels};

use std::time::Duration;
use std::error::Error;
use std::collections::HashMap;

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


}

impl PeerRequestHandler for InProcPeerCommunicator {
	fn send_vote_request(&self, destination_node_id: u64, request: VoteRequest)-> Result<VoteResponse, Box<Error>>  {
		trace!("Destination Node {} Sending request {:?}",destination_node_id, request);
		self.votes_channels[&destination_node_id].send_request(request)
	}
	fn send_append_entries_request(&self, destination_node_id: u64, request: AppendEntriesRequest) -> Result<AppendEntriesResponse, Box<Error>>  {
		trace!("Destination Node {} Sending request {:?}",destination_node_id, request);
		self.append_entries_channels[&destination_node_id].send_request(request)
	}
}

impl PeerRequestChannels for InProcPeerCommunicator{
	fn vote_request_rx(&self, node_id : u64) -> Receiver<VoteRequest> {
		self.votes_channels[&node_id].get_request_rx()
	}

	fn vote_response_tx(&self, node_id : u64) -> Sender<VoteResponse> {
		self.votes_channels[&node_id].get_response_tx()
	}

	fn append_entries_request_rx(&self, node_id : u64) -> Receiver<AppendEntriesRequest> {
		self.append_entries_channels[&node_id].get_request_rx()
	}

	fn append_entries_response_tx(&self, node_id : u64) -> Sender<AppendEntriesResponse> {
		self.append_entries_channels[&node_id].get_response_tx()
	}
}
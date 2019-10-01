use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use futures::{future};
use tower_grpc::{Request, Response};

use raft::{PeerRequestChannels, PeerRequestHandler, LogEntry, EntryContent, DataEntryContent, NewClusterConfigurationEntryContent};
use raft::{RaftError};

use crate::communication::duplex_channel::DuplexChannel;
use crate::communication::network::peer_communicator::grpc::generated::grpc_peer_communicator::{server, AppendEntriesRequest, VoteRequest, AppendEntriesResponse, VoteResponse};
use std::collections::HashMap;
use crate::communication::network::peer_communicator::client_requests::{vote_request, append_entries_request};

#[derive(Clone)]
pub struct NetworkPeerCommunicator {
	node_id : u64,
	timeout: Duration,
	host: String,
	votes_channels: HashMap<u64,DuplexChannel<raft::VoteRequest, raft::VoteResponse>>,
	append_entries_channels: HashMap<u64,DuplexChannel<raft::AppendEntriesRequest, raft::AppendEntriesResponse>>,
	lock : Arc<Mutex<bool>>
}

impl NetworkPeerCommunicator {
	pub fn new(host : String, node_id : u64, timeout : Duration, run_server: bool) -> NetworkPeerCommunicator {
		let votes_channels = HashMap::new();
		let append_entries_channels = HashMap::new();

		let comm = NetworkPeerCommunicator {
			node_id,
			timeout,
			host,
			votes_channels,
			append_entries_channels,
			lock: Arc::new(Mutex::new(true))
		};

//		append_entries_duplex_channel: DuplexChannel::new(format!("AppendEntries channel NodeId={}", node_id), timeout),
//		request_vote_duplex_channel: DuplexChannel::new(format!("RequestVote channel NodeId={}", node_id), timeout),

		if run_server {
			let comm_clone = comm.clone();
			let listen_address = comm_clone.get_address();
			thread::spawn(move || super::server::run_server(listen_address, comm_clone));
		}
		comm
	}


	pub fn get_address(&self) -> SocketAddr{
		self.host.parse().unwrap()
	}
}

impl PeerRequestChannels for NetworkPeerCommunicator {
	fn vote_request_rx(&self, node_id : u64) -> Receiver<raft::VoteRequest> {
		let _lock = self.lock.lock();

		self.votes_channels[&node_id].get_request_rx()
	}

	fn vote_response_tx(&self, node_id : u64) -> Sender<raft::VoteResponse> {
		let _lock = self.lock.lock();

		self.votes_channels[&node_id].get_response_tx()
	}

	fn append_entries_request_rx(&self, node_id : u64) -> Receiver<raft::AppendEntriesRequest> {
		let _lock = self.lock.lock();

		self.append_entries_channels[&node_id].get_request_rx()
	}

	fn append_entries_response_tx(&self, node_id : u64) -> Sender<raft::AppendEntriesResponse> {
		let _lock = self.lock.lock();

		self.append_entries_channels[&node_id].get_response_tx()
	}
}

impl PeerRequestHandler for NetworkPeerCommunicator{
	fn send_vote_request(&self, destination_node_id: u64, request: raft::VoteRequest) -> Result<raft::VoteResponse, RaftError> {
		let _lock = self.lock.lock();

		trace!("Destination Node {}. Vote request {:?}",destination_node_id, request);

		vote_request(self.host.clone(), self.timeout, request)
	}

	fn send_append_entries_request(&self, destination_node_id: u64, request: raft::AppendEntriesRequest) -> Result<raft::AppendEntriesResponse, RaftError> {
		let _lock = self.lock.lock();

		trace!("Destination Node {}. Append entries request {:?}",destination_node_id, request);

		append_entries_request(self.host.clone(), self.timeout, request)
	}
}

impl server::PeerRequestHandler for NetworkPeerCommunicator {
	type AppendEntriesFuture = future::FutureResult<Response<AppendEntriesResponse>, tower_grpc::Status>;
	type RequestVoteFuture = future::FutureResult<Response<VoteResponse>, tower_grpc::Status>;

	fn append_entries(&mut self, request: Request<AppendEntriesRequest>) -> Self::AppendEntriesFuture {
		let _lock = self.lock.lock();

		trace!("Append entries request {:?}", request);

		let raft_request = convert_append_entries_request(request);

		let send_result = self.append_entries_channels[&self.node_id].request_tx.send_timeout(raft_request, self.timeout);
		if let Err(err) = send_result {
			return future::err(tower_grpc::Status::new(tower_grpc::Code::Unknown, format!(" error:{}",err)))
		}

		let receive_result = self.append_entries_channels[&self.node_id].response_rx.recv_timeout(self.timeout);
		if let Err(err) = receive_result {
			return future::err(tower_grpc::Status::new(tower_grpc::Code::Unknown, format!(" error:{}",err)))
		}

		if let Ok(resp) = receive_result {
			let response = Response::new(AppendEntriesResponse{
				term: resp.term,
				success: resp.success
			});
			return future::ok(response);
		}

		unreachable!("invalid receive-response sequence");
	}

	fn request_vote(&mut self, request: Request<VoteRequest>) -> Self::RequestVoteFuture {
		let _lock = self.lock.lock();

		trace!("Vote request {:?}", request);

		let raft_request = raft::VoteRequest{
			term: request.get_ref().term,
			candidate_id: request.get_ref().candidate_id,
			last_log_term: request.get_ref().last_log_term,
			last_log_index: request.get_ref().last_log_index
		};

		let send_result = self.votes_channels[&self.node_id].request_tx.send_timeout(raft_request, self.timeout);
		if let Err(err) = send_result {
			return future::err(tower_grpc::Status::new(tower_grpc::Code::Unknown, format!(" error:{}",err)))
		}

		let receive_result = self.votes_channels[&self.node_id].response_rx.recv_timeout(self.timeout);
		if let Err(err) = receive_result {
			return future::err(tower_grpc::Status::new(tower_grpc::Code::Unknown, format!(" error:{}",err)))
		}

		if let Ok(resp) = receive_result {
			let response = Response::new(VoteResponse{
				term: resp.term,
				vote_granted: resp.vote_granted,
				peer_id: resp.peer_id
			});
			return future::ok(response);
		}

		unreachable!("invalid receive-response sequence");

	}
}


fn convert_append_entries_request(request: Request<AppendEntriesRequest>) -> raft::AppendEntriesRequest {
	let req = request.into_inner();
	let entries = req
		.entries
		.into_iter()
		.map(|entry| {
			let content = match entry.content_type {
				1 => {
					let inner_vec = entry.data;
					let data = inner_vec.into_boxed_slice();

					EntryContent::Data(DataEntryContent { data: Arc::new(Box::leak(data)) })
				},
				2 => EntryContent::AddServer(
					NewClusterConfigurationEntryContent { new_cluster_configuration: entry.new_cluster_configuration.clone() }),
				_ => panic!("invalid content type")
			};

			LogEntry {
				term: entry.term,
				index: entry.index,
				entry_content: content
			}
		})
		.collect();
	let raft_req = raft::AppendEntriesRequest {
		term: req.term,
		prev_log_term: req.prev_log_term,
		prev_log_index: req.prev_log_index,
		leader_id: req.leader_id,
		leader_commit: req.leader_commit,
		entries
	};

	raft_req
}


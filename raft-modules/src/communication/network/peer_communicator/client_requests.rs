use hyper::client::connect::{Destination, HttpConnector};
use tower_hyper::{client, util};
use tower_util::MakeService;
use futures::{Future};
use tower_grpc::{Request};

use raft::{EntryContent};
use std::time::Duration;
use raft::{new_err, RaftError};
use crate::communication::network::peer_communicator::grpc::generated::grpc_peer_communicator::client::PeerRequestHandler;
use crate::communication::network::peer_communicator::grpc::generated::grpc_peer_communicator::{AppendEntriesRequest, LogEntry, VoteRequest};


pub fn append_entries_request(host : String, timeout: Duration, request : raft::AppendEntriesRequest) -> Result<raft::AppendEntriesResponse, RaftError>{
	let uri = get_uri(host);
	let dst = Destination::try_from_uri(uri.clone()).expect("valid URI");

	let connector = util::Connector::new(HttpConnector::new(4));
	let settings = client::Builder::new().http2_only(true).clone();
	let mut make_client = client::Connect::with_builder(connector, settings);
	let (tx, rx)= crossbeam_channel::unbounded();
	let err_tx = tx.clone();

	let client_service = make_client
		.make_service(dst)
		.map_err(|e| {
			tower_grpc::Status::new(tower_grpc::Code::Unknown, format!("Connection error:{}",e))
		})
		.and_then(move |conn| {
			let conn = tower_request_modifier::Builder::new()
				.set_origin(uri)
				.build(conn)
				.expect("valid request builder");

			// Wait until the client is ready...
			PeerRequestHandler::new(conn).ready()
		});
	let request = client_service
		.and_then(move |mut client|
			{
			client.append_entries(Request::new(convert_append_entries_request(request)))
		});
	let response = request
		.and_then(move |response| {
			trace!("Append entries RESPONSE = {:?}", response);

			let resp = raft::AppendEntriesResponse{
				success: response.get_ref().success,
				term: response.get_ref().term
			};

			tx.send(Ok(resp)).expect("can send response");
			Ok(())
		})
		.map_err(move |e| {
			error!("Append entries request failed = {:?}", e);
			err_tx.send(Err(format!("Communication error:{}", e))).expect("can send error");
		});

	tokio::run(response);

	let receive_result = rx.recv_timeout(timeout);
	let result = receive_result.expect("valid response");
	match result {
		Ok(resp) => {Ok(resp) },
		Err(str) => { new_err(str, String::new()) },
	}

}

pub fn vote_request(host : String, timeout: Duration, request : raft::VoteRequest) -> Result<raft::VoteResponse, RaftError>{
	let uri = get_uri(host);
	let dst = Destination::try_from_uri(uri.clone()).expect("valid URI");

	let connector = util::Connector::new(HttpConnector::new(4));
	let settings = client::Builder::new().http2_only(true).clone();
	let mut make_client = client::Connect::with_builder(connector, settings);
	let (tx, rx)= crossbeam_channel::unbounded();
	let err_tx = tx.clone();

	let client_service = make_client
		.make_service(dst)
		.map_err(|e| {
			tower_grpc::Status::new(tower_grpc::Code::Unknown, format!("Connection error:{}",e))
		})
		.and_then(move |conn| {
			let conn = tower_request_modifier::Builder::new()
				.set_origin(uri)
				.build(conn)
				.expect("valid request builder");

			// Wait until the client is ready...
			PeerRequestHandler::new(conn).ready()
		});
	let request = client_service
		.and_then(move |mut client|
			{
			client.request_vote(Request::new(VoteRequest{
				term: request.term,
				candidate_id: request.candidate_id,
				last_log_term: request.last_log_term,
				last_log_index: request.last_log_index
			}))
		});
	let response = request
		.and_then(move |response| {
			trace!("Vote RESPONSE = {:?}", response);

			let resp = raft::VoteResponse{
				vote_granted: response.get_ref().vote_granted,
				term: response.get_ref().term,
				peer_id: response.get_ref().peer_id
			};

			tx.send(Ok(resp)).expect("can send response");
			Ok(())
		})
		.map_err(move |e| {
			error!("Append entries request failed = {:?}", e);
			err_tx.send(Err(format!("Communication error:{}", e))).expect("can send error");
		});

	tokio::run(response);

	let receive_result = rx.recv_timeout(timeout);
	let result = receive_result.expect("valid response");
	match result {
		Ok(resp) => {Ok(resp) },
		Err(str) => { new_err(str, String::new()) },
	}

}

fn convert_append_entries_request(request: raft::AppendEntriesRequest) -> AppendEntriesRequest {
	let entries = request
		.entries
		.into_iter()
		.map(|entry| {
			let (content_type, data, new_cluster_configuration) = match entry.entry_content {
				EntryContent::Data(content) 	 => (1,content.data.to_vec(), Vec::new()),
				EntryContent::AddServer(content) => (2,Vec::new(), content.new_cluster_configuration),
			};

			LogEntry{
				index: entry.index,
				term: entry.term,
				content_type,
				data,
				new_cluster_configuration
			}
		})
		.collect();
	
	AppendEntriesRequest{
		term: request.term,
		leader_commit: request.leader_commit,
		leader_id: request.leader_id,
		prev_log_index: request.prev_log_index,
		prev_log_term: request.prev_log_term,
		entries
	}
}


fn get_uri(host: String) -> http::Uri{
	format!("http://{}", host).parse().unwrap()
}



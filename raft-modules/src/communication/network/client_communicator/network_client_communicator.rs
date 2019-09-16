use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::error::Error;

use crossbeam_channel::{Receiver, Sender};
use futures::{future};
use tower_grpc::{Request, Response};

use raft::{ ClientRequestHandler};
use raft::{ ClientRequestChannels};

use crate::communication::network::client_communicator::grpc::generated::gprc_client_communicator::{server, ClientRpcResponse, AddServerRequest, NewDataRequest};
use super::client_requests::{new_data_request};
use super::server::run_server;
use crate::communication::duplex_channel::DuplexChannel;
use crate::communication::network::client_communicator::client_requests::add_server_request;

#[derive(Clone)]
pub struct NetworkClientCommunicator {
	node_id : u64,
	timeout: Duration,
	host: String,
	add_server_duplex_channel: DuplexChannel<raft::AddServerRequest, raft::ClientRpcResponse>,
	new_data_duplex_channel: DuplexChannel<raft::NewDataRequest, raft::ClientRpcResponse>
}

impl NetworkClientCommunicator {
	pub fn new(host : String, node_id : u64, timeout : Duration) -> NetworkClientCommunicator {
		let comm = NetworkClientCommunicator {
			node_id,
			timeout,
			host,
			add_server_duplex_channel: DuplexChannel::new(format!("AddServer channel NodeId={}", node_id), timeout),
			new_data_duplex_channel: DuplexChannel::new(format!("NewData channel NodeId={}", node_id), timeout)
		};

		let comm_clone = comm.clone();
		let listen_address = comm_clone.get_address();
		thread::spawn(move ||run_server(listen_address, comm_clone));

		comm
	}


	pub fn get_address(&self) -> SocketAddr{
		self.host.parse().unwrap()
	}
}

impl ClientRequestChannels for NetworkClientCommunicator {
	fn add_server_request_rx(&self) -> Receiver<raft::AddServerRequest> {
		self.add_server_duplex_channel.get_request_rx()
	}

	fn add_server_response_tx(&self) -> Sender<raft::ClientRpcResponse> {
		self.add_server_duplex_channel.get_response_tx()
	}

	fn new_data_request_rx(&self) -> Receiver<raft::NewDataRequest> {
		self.new_data_duplex_channel.get_request_rx()
	}

	fn new_data_response_tx(&self) -> Sender<raft::ClientRpcResponse> {
		self.new_data_duplex_channel.get_response_tx()
	}
}

impl ClientRequestHandler for NetworkClientCommunicator{
	fn add_server(&self, request: raft::AddServerRequest) -> Result<raft::ClientRpcResponse, Box<Error>> {
		trace!("Add server request {:?}", request);

		add_server_request(self.host.clone(), self.timeout, request)
	}


	fn new_data(&self, request: raft::NewDataRequest) -> Result<raft::ClientRpcResponse, Box<Error>> {
		trace!("New data request {:?}", request);

		new_data_request(self.host.clone(),self.timeout, request)
	}
}


impl server::ClientRequestHandler for NetworkClientCommunicator {
	type AddServerFuture = future::FutureResult<Response<ClientRpcResponse>, tower_grpc::Status>;
	type NewDataFuture = future::FutureResult<Response<ClientRpcResponse>, tower_grpc::Status>;

	fn add_server(&mut self, request: Request<AddServerRequest>) -> Self::AddServerFuture {
		trace!("Add server request {:?}", request);
		let raft_req = raft::AddServerRequest{new_server: request.into_inner().new_server};
		let send_result = self.add_server_duplex_channel.request_tx.send_timeout(raft_req, self.timeout);
		if let Err(err) = send_result {
			return future::err(tower_grpc::Status::new(tower_grpc::Code::Unknown, format!(" error:{}",err)))
		}

		let receive_result = self.add_server_duplex_channel.response_rx.recv_timeout(self.timeout);
		if let Err(err) = receive_result {
			return future::err(tower_grpc::Status::new(tower_grpc::Code::Unknown, format!(" error:{}",err)))
		}

		if let Ok(resp) = receive_result {
			let response = Response::new(ClientRpcResponse {
				current_leader: resp.current_leader.unwrap(),
				status: 1
			});

			return future::ok(response);
		}

		unreachable!("invalid receive-response sequence");

	}

	fn new_data(&mut self, request: Request<NewDataRequest>) -> Self::NewDataFuture {
		trace!("New data request {:?}", request);
		let inner_vec = request.into_inner().data;
		let data = inner_vec.into_boxed_slice();
		let raft_req = raft::NewDataRequest{data: Arc::new(Box::leak(data))};
		let send_result = self.new_data_duplex_channel.request_tx.send_timeout(raft_req, self.timeout);
		if let Err(err) = send_result {
			return future::err(tower_grpc::Status::new(tower_grpc::Code::Unknown, format!(" error:{}",err)))
		}

		let receive_result = self.new_data_duplex_channel.response_rx.recv_timeout(self.timeout);
		if let Err(err) = receive_result {
			return future::err(tower_grpc::Status::new(tower_grpc::Code::Unknown, format!(" error:{}",err)))
		}

		if let Ok(resp) = receive_result {
			let response = Response::new(ClientRpcResponse {
				current_leader: resp.current_leader.unwrap(),
				status: 1
			});

			return future::ok(response);
		}

		unreachable!("invalid receive-response sequence");
	}
}


use crate::communication::duplex_channel::DuplexChannel;
use ruft::{ ClientRequestHandler};
use ruft::{ ClientRequestChannels};
use std::time::Duration;
use crossbeam_channel::{Receiver, Sender};
use std::error::Error;
use crate::errors;

use futures::sync::mpsc;
use futures::{future, stream, Future, Sink, Stream};
use log::error;
use std::hash::{Hash, Hasher};
use std::time::Instant;
use tokio::net::TcpListener;
use tower_grpc::{Request, Response, Streaming};
use tower_hyper::server::{Http, Server};


use crate::network::client_communicator::grpc::generated::gprc_client_communicator::{server, ClientRpcResponse, AddServerRequest, NewDataRequest};
use crate::communication::comm_client::{new_data_request};
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;


#[derive(Clone)]
pub struct NetworkClientCommunicator {
	node_id : u64,
	timeout: Duration,
	add_server_duplex_channel: DuplexChannel<ruft::AddServerRequest, ruft::ClientRpcResponse>,
	new_data_duplex_channel: DuplexChannel<ruft::NewDataRequest, ruft::ClientRpcResponse>
}

impl NetworkClientCommunicator {
	pub fn new(node_id : u64, timeout : Duration) -> NetworkClientCommunicator {
		let comm = NetworkClientCommunicator {
			node_id,
			timeout,
			add_server_duplex_channel: DuplexChannel::new(format!("AddServer channel NodeId={}", node_id), timeout),
			new_data_duplex_channel: DuplexChannel::new(format!("NewData channel NodeId={}", node_id), timeout)
		};

		let comm_clone = comm.clone();
		thread::spawn(move ||comm_clone.run_server());

		comm
	}

	fn old_new_data(&self, request: ruft::NewDataRequest) -> Result<ruft::ClientRpcResponse, Box<Error>> {
		trace!("New data request {:?}", request);

		let send_result = self.new_data_duplex_channel.request_tx.send_timeout(request, self.timeout);
		if let Err(err) = send_result {
			return
				errors::new_err( format!("Cannot send request. Channel : {} ", "new_data"),  Some(Box::new(err)))

		}

		let receive_result = self.new_data_duplex_channel.response_rx.recv_timeout(self.timeout);
		if let Err(err) = receive_result {
			return errors::new_err(format!("Cannot receive response. Channel : {}", "new_data"), Some(Box::new(err)))
		}
		if let Ok(resp) = receive_result {
			let response = resp;

			return Ok(response);
		}

		unreachable!("invalid request-response sequence");
	}


//	pub fn run_server(handler: &NetworkClientCommunicator)
	pub fn run_server(&self)
	{
//	let _ = ::env_logger::init();

		let new_service = server::ClientRequestHandlerServer::new(self.clone());

		let mut server = Server::new(new_service);
		let http = Http::new().http2_only(true).clone();

		let addr = self.get_address();
		let bind = TcpListener::bind(&addr).expect("bind");

		println!("listening on {:?}", addr);

		let serve = bind
			.incoming()
			.for_each(move |sock| {
				if let Err(e) = sock.set_nodelay(true) {
					return Err(e);
				}

				let serve = server.serve_with(sock, http.clone());
				tokio::spawn(serve.map_err(|e| error!("h2 error: {:?}", e)));

				Ok(())
			})
			.map_err(|e| eprintln!("accept error: {}", e));

		tokio::run(serve);

	}

	pub fn get_address(&self) -> SocketAddr{
		format!("127.0.0.1:{}", 50000 + self.node_id).parse().unwrap()
	}
}

impl ClientRequestChannels for NetworkClientCommunicator {
	fn add_server_request_rx(&self) -> Receiver<ruft::AddServerRequest> {
		self.add_server_duplex_channel.get_request_rx()
	}

	fn add_server_response_tx(&self) -> Sender<ruft::ClientRpcResponse> {
		self.add_server_duplex_channel.get_response_tx()
	}

	fn new_data_request_rx(&self) -> Receiver<ruft::NewDataRequest> {
		self.new_data_duplex_channel.get_request_rx()
	}

	fn new_data_response_tx(&self) -> Sender<ruft::ClientRpcResponse> {
		self.new_data_duplex_channel.get_response_tx()
	}
}

impl ClientRequestHandler for NetworkClientCommunicator{
	fn add_server(&self, request: ruft::AddServerRequest) -> Result<ruft::ClientRpcResponse, Box<Error>> {
		unreachable!();
		trace!("Add server request {:?}", request);
		self.add_server_duplex_channel.send_request(request)
	}


	fn new_data(&self, request: ruft::NewDataRequest) -> Result<ruft::ClientRpcResponse, Box<Error>> {
		trace!("New data request {:?}", request);

		let resp = new_data_request(self.node_id, request);

		Ok(resp)
	}
}


impl server::ClientRequestHandler for NetworkClientCommunicator {
	type AddServerFuture = future::FutureResult<Response<ClientRpcResponse>, tower_grpc::Status>;
	type NewDataFuture = future::FutureResult<Response<ClientRpcResponse>, tower_grpc::Status>;

	fn add_server(&mut self, request: Request<AddServerRequest>) -> Self::AddServerFuture {
		let response = Response::new(ClientRpcResponse {
			current_leader : 12,
			status : 2
		});

		future::ok(response)
	}

	fn new_data(&mut self, request: Request<NewDataRequest>) -> Self::NewDataFuture {
		trace!("New data request {:?}", request);
		let mut vec = request.into_inner().data;

		let mut data = vec.into_boxed_slice();
		let ruft_req = ruft::NewDataRequest{data: Arc::new(Box::leak(data))};
		let send_result = self.new_data_duplex_channel.request_tx.send_timeout(ruft_req, self.timeout);
		if let Err(err) = send_result {
			return
			//	future::err(errors::new_err( format!("Cannot send request. Channel : {} ", "new_data"),  Some(Box::new(err))))
				future::err(tower_grpc::Status::new(tower_grpc::Code::Unknown, format!(" error:{}",err)))

		}

		let receive_result = self.new_data_duplex_channel.response_rx.recv_timeout(self.timeout);
		if let Err(err) = receive_result {
			return future::err(tower_grpc::Status::new(tower_grpc::Code::Unknown, format!(" error:{}",err)))
	//		return future::err(errors::new_err(format!("Cannot receive response. Channel : {}", "new_data"), Some(Box::new(err))))
		}

		if let Ok(resp) = receive_result {
			let response = Response::new(ClientRpcResponse {
				current_leader: resp.current_leader.unwrap(),
				status: 1
			});

			return future::ok(response);
		}

		unreachable!("unreach");
	}
}



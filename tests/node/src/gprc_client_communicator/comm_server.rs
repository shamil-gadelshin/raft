
use futures::sync::mpsc;
use futures::{future, stream, Future, Sink, Stream};
use log::error;
use std::hash::{Hash, Hasher};
use std::time::Instant;
use tokio::net::TcpListener;
use tower_grpc::{Request, Response, Streaming};
use tower_hyper::server::{Http, Server};

#[derive(Clone, Debug)]
struct ClientRequester;


use crate::gprc_client_communicator::grpc::gprc_client_communicator::{server,ClientRpcResponse, AddServerRequest, NewDataRequest};
use std::net::SocketAddr;


impl server::ClientRequestHandler for ClientRequester {
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
		let response = Response::new(ClientRpcResponse {
			current_leader : 0,
			status : 1
		});

		future::ok(response)
	}
}

pub fn run_server() {
//	let _ = ::env_logger::init();

	let node_id = 1u64;
	let new_service = server::ClientRequestHandlerServer::new(ClientRequester);

	let mut server = Server::new(new_service);
	let http = Http::new().http2_only(true).clone();

	let addr = get_address(node_id);
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

fn get_address(node_id: u64) -> SocketAddr{
	format!("127.0.0.1:{}", 50000 + node_id).parse().unwrap()
}

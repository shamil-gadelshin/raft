use futures::{Future, Stream};
use log::error;
use tokio::net::TcpListener;
use tower_hyper::server::{Http, Server};

use crate::communication::network::peer_communicator::grpc::generated::grpc_peer_communicator::{server};
use crate::NetworkPeerCommunicator;
use std::net::SocketAddr;
use crate::communication::network::peer_communicator::service_discovery::PeerCommunicatorServiceDiscovery;

pub fn run_server<Psd>(addr : SocketAddr, communicator : NetworkPeerCommunicator<Psd>)
	where Psd: PeerCommunicatorServiceDiscovery {
	let new_service = server::PeerRequestHandlerServer::new(communicator);

	let mut server = Server::new(new_service);
	let http = Http::new().http2_only(true).clone();

	let bind = TcpListener::bind(&addr).expect("can bind");

	info!("Network client communicator: listening on {:?}", addr);

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

use hyper::client::connect::{Destination, HttpConnector};
use tower_hyper::{client, util, Connection};
use tower_util::MakeService;
use futures::sync::mpsc;
use futures::{future, stream, Future, Sink, Stream};
use log::error;
use std::hash::{Hash, Hasher};
use std::time::Instant;
use tokio::net::TcpListener;
use tower_grpc::{Request, Response, Streaming, BoxBody, Status};
use tower_hyper::server::{Http, Server};
use tower_request_modifier::RequestModifier;
use crate::gprc_client_communicator::grpc::gprc_client_communicator::{ClientRpcResponse, AddServerRequest, NewDataRequest};
use crate::gprc_client_communicator::grpc::gprc_client_communicator::client::ClientRequestHandler;
use tower_hyper::util::Connector;
use hyper::client::connect::dns::GaiResolver;
use tower_hyper::client::ConnectFuture;
use http::Uri;
use std::error::Error;
use ruft::ClientResponseStatus;
use tokio::sync::oneshot;


pub fn add_server_request() {
	let uri = get_uri(1);
	let dst = Destination::try_from_uri(uri.clone()).expect("valid URI");

	let connector = util::Connector::new(HttpConnector::new(4));
	let settings = client::Builder::new().http2_only(true).clone();
	let mut make_client = client::Connect::with_builder(connector, settings);
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
			ClientRequestHandler::new(conn).ready()
		});
	let request = client_service
		.and_then(|mut client|
			{
			client.add_server(Request::new(AddServerRequest {
				new_server: 25
			}))
		});
	let response = request
		.and_then(|response| {
			println!("RESPONSE = {:?}", response);
			Ok(())
		})
		.map_err(|e| {
			println!("ERR = {:?}", e);
		});

	tokio::run(response);
}

pub fn new_data_request(node_id: u64, request : ruft::NewDataRequest) -> ruft::ClientRpcResponse {
	let uri = get_uri(node_id);
	println!("uri: {}", uri);
	let dst = Destination::try_from_uri(uri.clone()).expect("valid URI");

	let connector = util::Connector::new(HttpConnector::new(4));
	let settings = client::Builder::new().http2_only(true).clone();
	let mut make_client = client::Connect::with_builder(connector, settings);

	let (tx, rx) = crossbeam_channel::unbounded();
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
			ClientRequestHandler::new(conn).ready()
		});
	let request = client_service
		.and_then(move |mut client|
			{
			client.new_data(Request::new(NewDataRequest {
				data: Vec::from(*request.data)
			}))
		});
	let response = request
		.and_then(move |response| {
			println!("RESPONSE = {:?}", response);

			let resp = ruft::ClientRpcResponse{current_leader:Some(response.get_ref().current_leader), status : ClientResponseStatus::Ok};
			tx.send(resp);
			Ok(())
		})
		.map_err(|e| {
			println!("ERR = {:?}", e);
	//		tx.send(Err(Box::new(e)));
		});

	tokio::run(response);

	rx.recv().expect("receive result")//.expect("valid response")
}

fn get_uri(node_id: u64) -> http::Uri{
	format!("http://127.0.0.1:{}", 50000 + node_id).parse().unwrap()
}



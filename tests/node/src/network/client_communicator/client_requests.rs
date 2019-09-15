use hyper::client::connect::{Destination, HttpConnector};
use tower_hyper::{client, util};
use tower_util::MakeService;
use futures::{Future};
use tower_grpc::{Request};

use crate::network::client_communicator::grpc::generated::gprc_client_communicator::{AddServerRequest, NewDataRequest};
use crate::network::client_communicator::grpc::generated::gprc_client_communicator::client::ClientRequestHandler;

use ruft::ClientResponseStatus;


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
			tx.send(resp).expect("can send response");
			Ok(())
		})
		.map_err(|e| {
			println!("ERR = {:?}", e);
	//		tx.send(Err(Box::new(e)));
		});

	tokio::run(response);

	rx.recv().expect("receive result")
}

fn get_uri(node_id: u64) -> http::Uri{
	format!("http://127.0.0.1:{}", 50000 + node_id).parse().unwrap()
}



use hyper::client::connect::{Destination, HttpConnector};
use tower_hyper::{client, util};
use tower_util::MakeService;
use futures::{Future};
use tower_grpc::{Request};

use crate::communication::network::client_communicator::grpc::generated::gprc_client_communicator::{AddServerRequest, NewDataRequest};
use crate::communication::network::client_communicator::grpc::generated::gprc_client_communicator::client::ClientRequestHandler;

use ruft::{ClientResponseStatus};
use std::error::Error;
use std::time::Duration;
use crate::errors::new_err;


pub fn add_server_request(host : String, _timeout: Duration, request : ruft::AddServerRequest) -> Result<ruft::ClientRpcResponse, Box<Error>>{
	let uri = get_uri(host);
	let dst = Destination::try_from_uri(uri.clone()).expect("valid URI");

	let connector = util::Connector::new(HttpConnector::new(4));
	let settings = client::Builder::new().http2_only(true).clone();
	let mut make_client = client::Connect::with_builder(connector, settings);
	let (tx, rx)= crossbeam_channel::unbounded();//: Result<ClientRpcResponse, Box<Error>>
	let (err_tx, err_rx)= crossbeam_channel::unbounded();//: Result<ClientRpcResponse, Box<Error>>

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
			client.add_server(Request::new(AddServerRequest {
				new_server: request.new_server
			}))
		});
	let response = request
		.and_then(move |response| {
			trace!("NewData RESPONSE = {:?}", response);

			let resp = ruft::ClientRpcResponse{current_leader:Some(response.get_ref().current_leader), status : ClientResponseStatus::Ok};
			tx.send(resp).expect("can send response");
			Ok(())
		})
		.map_err(move |e| {
			error!("NewData request failed = {:?}", e);
			err_tx.send(format!("Communication error:{}", e)).expect("can send error");
		});

	tokio::run(response);


	crossbeam_channel::select!(
            recv(rx) -> resp  => {return Ok(resp.expect("valid response"))},
            recv(err_rx) -> err => {return new_err(err.expect("valid error"), None)},
    );

}

pub fn new_data_request(host: String, _timeout: Duration, request : ruft::NewDataRequest) -> Result<ruft::ClientRpcResponse, Box<Error>> {
	let uri = get_uri(host);
	let dst = Destination::try_from_uri(uri.clone()).expect("valid URI");

	let connector = util::Connector::new(HttpConnector::new(4));
	let settings = client::Builder::new().http2_only(true).clone();
	let mut make_client = client::Connect::with_builder(connector, settings);

	let (tx, rx)= crossbeam_channel::unbounded();//: Result<ClientRpcResponse, Box<Error>>
	let (err_tx, err_rx)= crossbeam_channel::unbounded();//: Result<ClientRpcResponse, Box<Error>>
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
			trace!("NewData RESPONSE = {:?}", response);

			let resp = ruft::ClientRpcResponse{current_leader:Some(response.get_ref().current_leader), status : ClientResponseStatus::Ok};
			tx.send(resp).expect("can send response");
			Ok(())
		})
		.map_err(move |e| {
			error!("NewData request failed = {:?}", e);
			err_tx.send(format!("Communication error:{}", e)).expect("can send error");
		});

	tokio::run(response);


	crossbeam_channel::select!(
            recv(rx) -> resp  => {return Ok(resp.expect("valid response"))},
            recv(err_rx) -> err => {return new_err(err.expect("valid error"), None)},
    );
}

fn get_uri(host: String) -> http::Uri{
	format!("http://{}", host).parse().unwrap()
}



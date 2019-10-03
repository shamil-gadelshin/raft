use futures::Future;
use hyper::client::connect::{Destination, HttpConnector};
use tower_grpc::Request;
use tower_hyper::{client, util};
use tower_util::MakeService;

use crate::communication::network::client_communicator::grpc::generated::grpc_client_communicator::{AddServerRequest, NewDataRequest, ClientRpcResponse};
use crate::communication::network::client_communicator::grpc::generated::grpc_client_communicator::client::ClientRequestHandler;

use raft::ClientResponseStatus;
use raft::{new_err, RaftError};
use std::time::Duration;

pub fn add_server_request(
    host: String,
    timeout: Duration,
    request: raft::AddServerRequest,
) -> Result<raft::ClientRpcResponse, RaftError> {
    let uri = get_uri(host);
    let dst = Destination::try_from_uri(uri.clone()).expect("valid URI");

    let connector = util::Connector::new(HttpConnector::new(4));
    let settings = client::Builder::new().http2_only(true).clone();
    let mut make_client = client::Connect::with_builder(connector, settings);
    let (tx, rx) = crossbeam_channel::unbounded();
    let err_tx = tx.clone();

    let client_service = make_client
        .make_service(dst)
        .map_err(|e| {
            tower_grpc::Status::new(tower_grpc::Code::Unknown, format!("Connection error:{}", e))
        })
        .and_then(move |conn| {
            let conn = tower_request_modifier::Builder::new()
                .set_origin(uri)
                .build(conn)
                .expect("valid request builder");

            // Wait until the client is ready...
            ClientRequestHandler::new(conn).ready()
        });
    let request = client_service.and_then(move |mut client| {
        client.add_server(Request::new(AddServerRequest {
            new_server: request.new_server,
        }))
    });
    let response = request
        .and_then(move |response| {
            trace!("NewData RESPONSE = {:?}", response);

            let resp = convert_response(response.get_ref());

            tx.send(Ok(resp)).expect("can send response");
            Ok(())
        })
        .map_err(move |e| {
            error!("NewData request failed = {}", e);
            err_tx
                .send(Err(format!("Communication error:{}", e)))
                .expect("can send error");
        });

    tokio::run(response);

    let receive_result = rx.recv_timeout(timeout);
    let result = receive_result.expect("valid response");
    match result {
        Ok(resp) => Ok(resp),
        Err(str) => new_err(str, String::new()),
    }
}

fn convert_response(grpc_response: &ClientRpcResponse) -> raft::ClientRpcResponse {
    let mut current_leader: Option<u64> = None;
    let response_current_leader = grpc_response.current_leader;
    if response_current_leader > 0 {
        current_leader = Some(response_current_leader);
    }

    let status = match grpc_response.status {
        1 => ClientResponseStatus::Ok,
        2 => ClientResponseStatus::NotLeader,
        3 => ClientResponseStatus::NoQuorum,
        4 => ClientResponseStatus::Error,
        _ => panic!("invalid client response status"),
    };

    raft::ClientRpcResponse {
        current_leader,
        status,
        message: grpc_response.message.clone(),
    }
}

pub fn new_data_request(
    host: String,
    timeout: Duration,
    request: raft::NewDataRequest,
) -> Result<raft::ClientRpcResponse, RaftError> {
    let uri = get_uri(host);
    let dst = Destination::try_from_uri(uri.clone()).expect("valid URI");

    let connector = util::Connector::new(HttpConnector::new(4));
    let settings = client::Builder::new().http2_only(true).clone();
    let mut make_client = client::Connect::with_builder(connector, settings);

    let (tx, rx) = crossbeam_channel::unbounded();
    let err_tx = tx.clone();
    let client_service = make_client
        .make_service(dst)
        .map_err(|e| {
            tower_grpc::Status::new(tower_grpc::Code::Unknown, format!("Connection error:{}", e))
        })
        .and_then(move |conn| {
            let conn = tower_request_modifier::Builder::new()
                .set_origin(uri)
                .build(conn)
                .expect("valid request builder");

            // Wait until the client is ready...
            ClientRequestHandler::new(conn).ready()
        });
    let request = client_service.and_then(move |mut client| {
        client.new_data(Request::new(NewDataRequest {
            data: Vec::from(*request.data),
        }))
    });
    let response = request
        .and_then(move |response| {
            trace!("NewData RESPONSE = {:?}", response);

            let resp = convert_response(response.get_ref());

            tx.send(Ok(resp)).expect("can send response");
            Ok(())
        })
        .map_err(move |e| {
            error!("NewData request failed = {}", e);
            err_tx
                .send(Err(format!("Communication error:{}", e)))
                .expect("can send error");
        });

    tokio::run(response);

    let receive_result = rx.recv_timeout(timeout);
    let result = receive_result.expect("valid response");
    match result {
        Ok(resp) => Ok(resp),
        Err(str) => new_err(str, String::new()),
    }
}

fn get_uri(host: String) -> http::Uri {
    format!("http://{}", host).parse().unwrap()
}

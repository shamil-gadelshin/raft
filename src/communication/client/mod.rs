use crossbeam_channel::{Sender, Receiver};

use crate::core::*;
use std::time::Duration;

#[derive(Clone)]
pub struct ClientRequestHandler {
    add_server_request_tx: Sender<AddServerRequest>,
    add_server_request_rx: Receiver<AddServerRequest>,
    add_server_response_tx: Sender<AddServerResponse>,
    add_server_response_rx: Receiver<AddServerResponse>,
}

impl ClientRequestHandler {
    pub fn new() -> ClientRequestHandler {
        let (add_server_request_tx, add_server_request_rx): (Sender<AddServerRequest>, Receiver<AddServerRequest>) = crossbeam_channel::bounded(0);
        let (add_server_response_tx, add_server_response_rx): (Sender<AddServerResponse>, Receiver<AddServerResponse>) = crossbeam_channel::bounded(0);

        let client = ClientRequestHandler{
            add_server_request_tx,
            add_server_request_rx,
            add_server_response_tx,
            add_server_response_rx
        };

        client
    }

    pub fn get_add_server_request_rx(&self) -> Receiver<AddServerRequest> {
        self.add_server_request_rx.clone()
    }

    pub fn get_add_server_response_tx(&self) -> Sender<AddServerResponse> {
        self.add_server_response_tx.clone()
    }

    //TODO change result error type
    pub fn add_server(&self, request: AddServerRequest) -> Result<AddServerResponse, &'static str> {
        print_event( format!("Add server request {:?}", request));

        let timeout = crossbeam_channel::after(Duration::new(1,0));
        select!(
            recv(timeout) -> _  => {
                return Err("Send add_server_request timeout")
 //TODO               return Err(format!("Send add_rpc_timeout - Destination Node {:?} Sending request {:?}",destination_node_id, request))
            },
            send(self.add_server_request_tx, request) -> res => {
                if let Err(err) = res {
                    return Err("Cannot send add_server_request")
                }
            },
        );

        select!(
            recv(timeout) -> _  => {
                return Err("Receive add_server_response timeout")
            },
            recv(self.add_server_response_rx) -> res => {
                if let Err(err) = res {
                    return Err("Cannot receive add_server_response")
                }
                if let Ok(resp) = res {
                    let add_server_response = resp;

                    return Ok(add_server_response);
                }
            },
        );

        panic!("invalid add_server request-response sequence");
    }
}


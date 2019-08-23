use crossbeam_channel::{Sender, Receiver};
use std::collections::HashMap;

use crate::core::*;
use std::error::Error;
use std::time::Duration;

#[derive(Clone)]
pub struct ClientRequestHandler {
    add_server_request_tx: Sender<AddServerRequest>,
    add_server_request_rx: Receiver<AddServerRequest>,
}

impl ClientRequestHandler {
    pub fn new() -> ClientRequestHandler {
        let (add_server_request_tx, add_server_request_rx): (Sender<AddServerRequest>, Receiver<AddServerRequest>) = crossbeam_channel::unbounded();

        let client = ClientRequestHandler{
            add_server_request_tx,
            add_server_request_rx,
        };

        client
    }

    pub fn get_add_server_channel_rx(&self) -> Receiver<AddServerRequest> {
        self.add_server_request_rx.clone()
    }

    pub fn add_server(&self, request: AddServerRequest) -> Result<AddServerResponse, &'static str> {
//        print_event( format!("Add server request {:?}", request));
//
//        let timeout = crossbeam_channel::after(Duration::new(1,0));
//        select!(
//            recv(timeout) -> _  => {
//                return Err("Send add_rpc_timeout")
// //TODO               return Err(format!("Send add_rpc_timeout - Destination Node {:?} Sending request {:?}",destination_node_id, request))
//            },
//            send(self.add_server_rpc_request_channels_tx[&destination_node_id], request) -> res => {
//
//            },
//
//            //TODO : notify leadership
//        );


        Ok(AddServerResponse{status:ChangeMembershipResponseStatus::Ok, current_leader:Some(1)})
        //      self.add_rpc_request_channels_tx[&destination_node_id].send(request).expect("cannot send vote request");
    }
}


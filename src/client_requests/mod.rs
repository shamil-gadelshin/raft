use crossbeam_channel::{Sender, Receiver};
use std::collections::HashMap;

use crate::core::*;
use std::error::Error;
use std::time::Duration;

#[derive(Clone)]
pub struct ClientHandler {
    add_server_rpc_request_channels_tx: HashMap<u64, Sender<AddServerRequest>>,
    add_server_rpc_request_channels_rx: HashMap<u64, Receiver<AddServerRequest>>,
}

impl ClientHandler {
    pub fn new(nodes : Vec<u64>) -> ClientHandler {
        let mut add_server_rpc_request_channels_tx = HashMap::new();
        let mut add_server_rpc_request_channels_rx = HashMap::new();

        let mut client = ClientHandler{
            add_server_rpc_request_channels_rx,
            add_server_rpc_request_channels_tx,
        };

        for node_id in nodes {
            communicator.add_node_communication(node_id);
        }

        client
    }


    pub fn add_node_communication(&mut self, node_id : u64) {
        let (add_server_request_tx, add_server_request_rx): (Sender<AddServerRequest>, Receiver<AddServerRequest>) = crossbeam_channel::unbounded();
        let (add_server_response_tx, add_server_response_rx): (Sender<AddServerResponse>, Receiver<AddServerResponse>) = crossbeam_channel::unbounded();

        self.add_server_rpc_request_channels_tx.insert(node_id, add_server_request_tx);
        self.add_server_rpc_request_channels_rx.insert(node_id, add_server_request_rx);
    }

    pub fn add_server(&self, request: AddServerRequest) -> Result<AddServerResponse, &'static str> {
        print_event( format!("Add server request {:?}", request));

        let timeout = crossbeam_channel::after(Duration::new(1,0));
        select!(
            recv(timeout) -> _  => {
                return Err("Send add_rpc_timeout")
 //TODO               return Err(format!("Send add_rpc_timeout - Destination Node {:?} Sending request {:?}",destination_node_id, request))
            },
            send(self.add_server_rpc_request_channels_tx[&destination_node_id], request) -> res => {

            },

            //TODO : notify leadership
        );


        Ok(AddServerResponse{status:ChangeMembershipResponseStatus::Ok, current_leader:Some(1)})
        //      self.add_rpc_request_channels_tx[&destination_node_id].send(request).expect("cannot send vote request");
    }
}


use crossbeam_channel::{Sender, Receiver};
use std::collections::HashMap;

use crate::core::*;
use std::error::Error;
use std::time::Duration;

#[derive(Clone)]
pub struct InProcNodeCommunicator {
    pub vote_request_channels_tx: HashMap<u64, Sender<VoteRequest>>,
    pub vote_response_channels_tx: HashMap<u64, Sender<VoteResponse>>,
    pub append_entries_request_channels_tx: HashMap<u64, Sender<AppendEntriesRequest>>,
    pub add_rpc_request_channels_tx: HashMap<u64, Sender<AddServerRequest>>,
    pub add_rpc_request_channels_rx: HashMap<u64, Receiver<AddServerRequest>>,
}

impl InProcNodeCommunicator {
    pub fn send_vote_request(&self, destination_node_id: u64, request: VoteRequest) {
        print_event( format!("Destination Node {:?} Sending request {:?}",destination_node_id, request));
        self.vote_request_channels_tx[&destination_node_id].send(request).expect("cannot send vote request");
    }
    pub fn send_vote_response(&self, destination_node_id: u64, response: VoteResponse) {
        print_event( format!("Destination Node {:?} Sending response {:?}", destination_node_id, response));
        self.vote_response_channels_tx[&destination_node_id].send(response).expect("cannot send vote response");
    }
    pub fn send_append_entries_request(&self, destination_node_id: u64, request: AppendEntriesRequest) {
        print_event( format!("Destination Node {:?} Sending request {:?}",destination_node_id, request));
        self.append_entries_request_channels_tx[&destination_node_id].send(request).expect("cannot send vote request");
    }

    pub fn add_server(&self, destination_node_id: u64, request: AddServerRequest) -> Result<AddServerResponse, &'static str> {
        print_event( format!("Destination Node {:?} Sending request {:?}",destination_node_id, request));

        let timeout = crossbeam_channel::after(Duration::new(1,0));
        select!(
            recv(timeout) -> _  => {
                return Err("Send add_rpc_timeout")
 //TODO               return Err(format!("Send add_rpc_timeout - Destination Node {:?} Sending request {:?}",destination_node_id, request))
            },
            send(self.add_rpc_request_channels_tx[&destination_node_id], request) -> res => {

            },

            //TODO : notify leadership
        );


        Ok(AddServerResponse{status:ChangeMembershipResponseStatus::Ok, current_leader:Some(1)})
  //      self.add_rpc_request_channels_tx[&destination_node_id].send(request).expect("cannot send vote request");
    }
}


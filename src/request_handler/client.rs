use std::sync::{Arc, Mutex};

use crossbeam_channel::Sender;

use crate::state::{Node, NodeStatus};
use crate::communication::client::{ClientRpcResponse, ClientResponseStatus, ClientRequestChannels};
use crate::operation_log::{LogStorage};
use crate::common::{AddServerEntryContent, EntryContent, DataEntryContent};
use crate::errors;
use crate::fsm::Fsm;
use std::error::Error;


pub struct ClientRequestHandlerParams<Log, FsmT, Cc : ClientRequestChannels>
	where Log: Sync + Send + LogStorage + 'static, FsmT: Sync + Send + Fsm + 'static {
	pub protected_node : Arc<Mutex<Node<Log, FsmT>>>,
	pub client_communicator : Cc
}


pub fn process_client_requests<Log: Sync + Send + LogStorage, FsmT:  Sync + Send + Fsm, Cc : ClientRequestChannels>(params : ClientRequestHandlerParams<Log,FsmT, Cc>) {
	let add_server_request_rx = params.client_communicator.add_server_request_rx();
	let new_data_request_rx = params.client_communicator.new_data_request_rx();
	loop {
		let response_tx;
		let entry_content;
		select!(
            recv(add_server_request_rx) -> res => {
				let request = res.expect("can get add server request");
				entry_content = EntryContent::AddServer(AddServerEntryContent { new_server: request.new_server });
				response_tx = params.client_communicator.add_server_response_tx();

           },
            recv(new_data_request_rx) -> res => {
            	let request = res.expect("can get new_data request");
				entry_content = EntryContent::Data(DataEntryContent { data: request.data });
				response_tx = params.client_communicator.new_data_response_tx();
            },
        );


		let client_rpc_response_result = process_client_request_internal(params.protected_node.clone(), entry_content);

		let process_request_result = match client_rpc_response_result {
			Ok(client_rpc_response) => {
				send_client_response(client_rpc_response, response_tx)
			},
			Err(err) => {
				error!("Process client request error: {}", err.description());
				Err(err)
			}
		};

		trace!("Client request processed: {:?}", process_request_result)
	}
}


//TODO inline
fn send_client_response(client_rpc_response: ClientRpcResponse, response_tx: Sender<ClientRpcResponse>) -> Result<(), Box<Error>> {
	let send_result = response_tx.send(client_rpc_response);
	if let Err(err) = send_result {
		return errors::new_err("cannot send clientRpcResponse".to_string(), Some(Box::new(err)))
	}

	Ok(())
}

fn process_client_request_internal<Log, FsmT>(protected_node: Arc<Mutex<Node<Log, FsmT>>>, entry_content: EntryContent) -> Result<ClientRpcResponse, Box<Error>>
where Log: Sync + Send + LogStorage, FsmT:  Sync + Send + Fsm {
	let mut node = protected_node.lock().expect("node lock is not poisoned");

	let client_rpc_response = match node.status {
		NodeStatus::Leader => {
			node.append_content_to_log(entry_content)?;
			ClientRpcResponse { status: ClientResponseStatus::Ok, current_leader: node.current_leader_id }
		},
		NodeStatus::Candidate | NodeStatus::Follower => {
			ClientRpcResponse { status: ClientResponseStatus::NotLeader, current_leader: node.current_leader_id }
		}
	};

	Ok(client_rpc_response)
}

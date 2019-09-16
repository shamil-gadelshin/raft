use std::sync::{Arc, Mutex};
use std::error::Error;

use crate::state::{Node, NodeStatus};
use crate::communication::client::{ClientRpcResponse, ClientResponseStatus, ClientRequestChannels};
use crate::operation_log::{LogStorage};
use crate::common::{AddServerEntryContent, EntryContent, DataEntryContent};
use crate::errors;
use crate::fsm::Fsm;
use crate::communication::peers::PeerRequestHandler;


pub struct ClientRequestHandlerParams<Log, FsmT, Cc : ClientRequestChannels,Pc>
	where Log: Sync + Send + LogStorage + 'static, FsmT: Sync + Send + Fsm + 'static, Pc : PeerRequestHandler + Clone {
	pub protected_node : Arc<Mutex<Node<Log, FsmT,Pc>>>,
	pub client_communicator : Cc
}


pub fn process_client_requests<Log: Sync + Send + LogStorage, FsmT:  Sync + Send + Fsm, Cc : ClientRequestChannels, Pc : PeerRequestHandler + Clone>(params : ClientRequestHandlerParams<Log,FsmT, Cc,Pc>) {
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
				let send_result = response_tx.send(client_rpc_response);
				if let Err(err) = send_result {
					errors::new_err("cannot send clientRpcResponse".to_string(), Some(Box::new(err)))
				} else {
					Ok(())
				}
			},
			Err(err) => {
				error!("Process client request error: {}", err.description());
				Err(err)
			}
		};

		trace!("Client request processed: {:?}", process_request_result)
	}
}

fn process_client_request_internal<Log, FsmT, Pc>(protected_node: Arc<Mutex<Node<Log, FsmT,Pc>>>, entry_content: EntryContent) -> Result<ClientRpcResponse, Box<Error>>
where Log: Sync + Send + LogStorage, FsmT:  Sync + Send + Fsm, Pc : PeerRequestHandler + Clone {
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

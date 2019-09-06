use std::sync::{Arc, Mutex};

use crate::state::{Node, NodeStatus};
use crate::communication::client::{AddServerResponse, ClientResponseStatus, InProcClientCommunicator, AddServerRequest, NewDataRequest, NewDataResponse};
use crate::operation_log::{LogStorage};
use crate::common::{AddServerEntryContent, EntryContent, DataEntryContent};
use crate::errors;
use crate::fsm::Fsm;

pub struct ClientRequestHandlerParams<Log, FsmT>
	where Log: Sync + Send + LogStorage + 'static, FsmT: Sync + Send + Fsm + 'static {
	pub protected_node : Arc<Mutex<Node<Log, FsmT>>>,
	pub client_communicator : InProcClientCommunicator
}


pub fn process_client_requests<Log: Sync + Send + LogStorage, FsmT:  Sync + Send + Fsm>(params : ClientRequestHandlerParams<Log,FsmT>) {
	let add_server_request_rx = params.client_communicator.get_add_server_request_rx();
	let new_data_request_rx = params.client_communicator.get_new_data_request_rx();
	loop {
		let process_request_result;
		select!(
            recv(add_server_request_rx) -> res => {
				process_request_result =process_add_server_request(params.protected_node.clone(), params.client_communicator.clone(), res.expect("can get add_server request"));
            },
            recv(new_data_request_rx) -> res => {
				process_request_result = process_new_data_request(params.protected_node.clone(), params.client_communicator.clone(), res.expect("can get new_data request"));
            },
        );

		trace!("Client request processed: {:?}", process_request_result)
	}
}

fn process_new_data_request<Log: Sync + Send + LogStorage, FsmT:  Sync + Send + Fsm>(protected_node: Arc<Mutex<Node<Log, FsmT>>>, client_communicator: InProcClientCommunicator, request: NewDataRequest) -> errors::Result<()>{
	let mut node = protected_node.lock().expect("node lock is not poisoned");

	info!("Node {:?} Received 'New data Request (Node {:?})'", node.id, request);
	let new_server_response = match node.status {
		NodeStatus::Leader => {
			let entry_content = EntryContent::Data(DataEntryContent { data: request.data });

			node.append_content_to_log(entry_content)?;
			NewDataResponse { status: ClientResponseStatus::Ok, current_leader: node.current_leader_id }
		},
		_ => {
			NewDataResponse { status: ClientResponseStatus::NotLeader, current_leader: node.current_leader_id }
		}
	};
	let send_result = client_communicator.get_new_data_response_tx().send(new_server_response);
	if let Err(err) = send_result {
		return errors::new_err("can send response new_data".to_string(), Some(Box::new(err)))
	}

	Ok(())
}

fn process_add_server_request<Log: Sync + Send + LogStorage, FsmT:  Sync + Send + Fsm>(protected_node: Arc<Mutex<Node<Log, FsmT>>>, client_communicator: InProcClientCommunicator, request: AddServerRequest) -> errors::Result<()>{
	let mut node = protected_node.lock().expect("node lock is not poisoned");

	info!("Node {:?} Received 'Add Server Request (Node {:?})' {:?}", node.id, request.new_server, request);
	let add_server_response = match node.status {
		NodeStatus::Leader => {
			let entry_content = EntryContent::AddServer(AddServerEntryContent { new_server: request.new_server });

			node.append_content_to_log(entry_content)?;

			AddServerResponse { status: ClientResponseStatus::Ok, current_leader: node.current_leader_id }
		},
		_ => {
			AddServerResponse { status: ClientResponseStatus::NotLeader, current_leader: node.current_leader_id }
		}
	};

	let send_result = client_communicator.get_add_server_response_tx().send(add_server_response);
	if let Err(err) = send_result {
		return errors::new_err("can send response client_add_server".to_string(), Some(Box::new(err)))
	}

	Ok(())
}

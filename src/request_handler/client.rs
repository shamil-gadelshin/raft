use std::sync::{Arc, Mutex};

use crate::state::{Node, NodeStatus};
use crate::communication::client::{AddServerResponse, ClientResponseStatus, InProcClientCommunicator, AddServerRequest, NewDataRequest, NewDataResponse};
use crate::operation_log::storage::{LogStorage};
use crate::common::{AddServerEntryContent, EntryContent, DataEntryContent};

pub fn process_client_requests<Log: Sync + Send + LogStorage>(protected_node: Arc<Mutex<Node<Log>>>,
														client_communicator : InProcClientCommunicator) {
	let add_server_request_rx = client_communicator.get_add_server_request_rx();
	let new_data_request_rx = client_communicator.get_new_data_request_rx();
	loop {
		select!(
            recv(add_server_request_rx) -> res => {
				process_add_server_request(protected_node.clone(), client_communicator.clone(), res.expect("can get add_server request"));
            },
            recv(new_data_request_rx) -> res => {
				process_new_data_request(protected_node.clone(), client_communicator.clone(), res.expect("can get new_data request"));
            },
        );
	}
}

fn process_new_data_request<Log: Sync + Send + LogStorage>(protected_node: Arc<Mutex<Node<Log>>>, client_communicator: InProcClientCommunicator, request: NewDataRequest) {
	let mut node = protected_node.lock().expect("node lock is not poisoned");

	info!("Node {:?} Received 'New data Request (Node {:?})'", node.id, request);
	let new_server_response = match node.status {
		NodeStatus::Leader => {
			let entry_content = EntryContent::Data(DataEntryContent { data: request.data });

			node.append_content_to_log(entry_content);
			NewDataResponse { status: ClientResponseStatus::Ok, current_leader: node.current_leader_id }
		},
		_ => {
			NewDataResponse { status: ClientResponseStatus::NotLeader, current_leader: node.current_leader_id }
		}
	};
	client_communicator.get_new_data_response_tx().send(new_server_response)
		.expect("can send response client_add_server");
}

fn process_add_server_request<Log: Sync + Send + LogStorage>(protected_node: Arc<Mutex<Node<Log>>>, client_communicator: InProcClientCommunicator, request: AddServerRequest) {
	let mut node = protected_node.lock().expect("node lock is not poisoned");

	info!("Node {:?} Received 'Add Server Request (Node {:?})' {:?}", node.id, request.new_server, request);
	let add_server_response = match node.status {
		NodeStatus::Leader => {
			let entry_content = EntryContent::AddServer(AddServerEntryContent { new_server: request.new_server });

			node.append_content_to_log(entry_content);

			AddServerResponse { status: ClientResponseStatus::Ok, current_leader: node.current_leader_id }
		},
		_ => {
			AddServerResponse { status: ClientResponseStatus::NotLeader, current_leader: node.current_leader_id }
		}
	};
	client_communicator.get_add_server_response_tx().send(add_server_response)
		.expect("can send response client_add_server");
}

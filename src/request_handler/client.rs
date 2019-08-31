use std::sync::{Arc, Mutex};

use crate::state::{Node, NodeStatus};
use crate::communication::client::{AddServerResponse, ChangeMembershipResponseStatus, InProcClientCommunicator};
use crate::operation_log::storage::{LogStorage};
use crate::common::{AddServerEntryContent, EntryContent};

pub fn process_client_requests<Log: Sync + Send + LogStorage>(mutex_node: Arc<Mutex<Node<Log>>>,
														client_communicator : InProcClientCommunicator) {
	loop {
		let request = client_communicator.get_add_server_request_rx().recv().expect("cannot get request from client_add_server_request_rx");

		let (node_id, node_status, current_leader_id) = {
			let node = mutex_node.lock().expect("node lock is not poisoned");

			(node.id, node.status, node.current_leader_id)
		};

		info!("Node {:?} Received 'Add Server Request (Node {:?})' {:?}", node_id, request.new_server, request);

		let add_server_response = match node_status{
			NodeStatus::Leader => {
				let entry_content = EntryContent::AddServer(AddServerEntryContent{new_server:request.new_server});

				let mut node = mutex_node.lock().expect("node lock is not poisoned");
				node.append_content_to_log(entry_content);

				AddServerResponse{status: ChangeMembershipResponseStatus::Ok, current_leader:current_leader_id}
			},
			_ => {
				AddServerResponse{status: ChangeMembershipResponseStatus::NotLeader, current_leader:current_leader_id}
			}
		};

		client_communicator.get_add_server_response_tx().send(add_server_response)
			.expect("can send response client_add_server");
	}
}

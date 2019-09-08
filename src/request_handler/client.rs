use std::sync::{Arc, Mutex};

use crossbeam_channel::Sender;

use crate::state::{Node, NodeStatus};
use crate::communication::client::{ClientRpcResponse, ClientResponseStatus, InProcClientCommunicator};
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
				let request = res.expect("can get add server request");
				let entry_content = EntryContent::AddServer(AddServerEntryContent { new_server: request.new_server });
					process_request_result = process_client_request(params.protected_node.clone(),
					params.client_communicator.get_add_server_response_tx(),
					entry_content);            },
            recv(new_data_request_rx) -> res => {
            	let request = res.expect("can get new_data request");
				let entry_content = EntryContent::Data(DataEntryContent { data: request.data });
					process_request_result = process_client_request(params.protected_node.clone(),
					params.client_communicator.get_new_data_response_tx(),
					entry_content);
            },
        );

		trace!("Client request processed: {:?}", process_request_result)
	}
}



fn process_client_request<Log, FsmT>(protected_node: Arc<Mutex<Node<Log, FsmT>>>, response_tx: Sender<ClientRpcResponse>, entry_content: EntryContent) -> errors::Result<()>
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
	let send_result = response_tx.send(client_rpc_response);
	if let Err(err) = send_result {
		return errors::new_err("can send response".to_string(), Some(Box::new(err)))
	}

	Ok(())
}

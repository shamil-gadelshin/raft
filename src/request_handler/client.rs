use std::sync::{Arc, Mutex};

use crate::state::{Node, NodeStatus, NodeStateSaver};
use crate::communication::client::{ClientRpcResponse, ClientResponseStatus, ClientRequestChannels};
use crate::operation_log::{OperationLog};
use crate::common::{NewClusterConfigurationEntryContent, EntryContent, DataEntryContent};
use crate::{errors, Cluster};
use crate::rsm::ReplicatedStateMachine;
use crate::communication::peers::PeerRequestHandler;
use crossbeam_channel::Receiver;
use crate::errors::RaftError;


pub struct ClientRequestHandlerParams<Log, Rsm, Cc,Pc, Ns, Cl>
	where Log: OperationLog,
		  Rsm: ReplicatedStateMachine,
		  Pc : PeerRequestHandler,
		  Cc : ClientRequestChannels,
		  Ns : NodeStateSaver,
		  Cl : Cluster
{	pub protected_node : Arc<Mutex<Node<Log, Rsm,Pc, Ns, Cl>>>,
	pub client_communicator : Cc,
	pub cluster_configuration: Cl
}


pub fn process_client_requests<Log, Rsm, Cc, Pc, Ns, Cl>(params : ClientRequestHandlerParams<Log,Rsm, Cc, Pc, Ns, Cl>,
													terminate_worker_rx : Receiver<()>)
	where Log: OperationLog,
		  Rsm: ReplicatedStateMachine,
		  Pc : PeerRequestHandler,
		  Cc : ClientRequestChannels,
		  Ns : NodeStateSaver,
		  Cl : Cluster{
	info!("Client request processor worker started");
	let add_server_request_rx = params.client_communicator.add_server_request_rx();
	let new_data_request_rx = params.client_communicator.new_data_request_rx();
	loop {
		let response_tx;
		let entry_content;
		select!(
			recv(terminate_worker_rx) -> res  => {
                if res.is_err() {
                    error!("Abnormal exit for client request processor worker");
                }
                break
            },
            recv(add_server_request_rx) -> res => {
				let request = res.expect("can get add server request");
				entry_content = EntryContent::AddServer(NewClusterConfigurationEntryContent {
					 new_cluster_configuration: get_new_configuration(params.cluster_configuration.clone(), request.new_server)
				 });
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
					errors::new_err("cannot send clientRpcResponse".to_string(), err.to_string())
				} else {
					Ok(())
				}
			},
			Err(err) => {
				let msg = format!("Process client request error: {}", err);
				error!("{}", msg);
				errors::new_err("Cannot send clientRpcResponse".to_string(), msg)
			}
		};

		trace!("Client request processed: {:?}", process_request_result)
	}
	info!("Client request processor worker stopped");
}

fn process_client_request_internal<Log, Rsm, Pc, Ns, Cl>(
	protected_node: Arc<Mutex<Node<Log, Rsm,Pc, Ns, Cl>>>,
	entry_content: EntryContent) -> Result<ClientRpcResponse, RaftError>
	where Log: OperationLog,
		  Rsm: ReplicatedStateMachine,
		  Pc : PeerRequestHandler,
		  Ns : NodeStateSaver,
		  Cl : Cluster{
	let mut node = protected_node.lock().expect("node lock is not poisoned");

	let client_rpc_response = match node.status {
		NodeStatus::Leader => {
			node.append_content_to_log(entry_content)?;
			ClientRpcResponse {
				status: ClientResponseStatus::Ok,
				current_leader: node.current_leader_id
			}
		},
		NodeStatus::Candidate | NodeStatus::Follower => {
			ClientRpcResponse {
				status: ClientResponseStatus::NotLeader,
				current_leader: node.current_leader_id
			}
		}
	};

	Ok(client_rpc_response)
}

fn get_new_configuration<Cl>(cluster_configuration: Cl, new_server: u64) -> Vec<u64>
	where Cl : Cluster{

	let mut nodes = cluster_configuration.get_all_nodes();
	nodes.push(new_server);

	nodes
}
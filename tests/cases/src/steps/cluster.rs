use raft_modules::{InProcPeerCommunicator, NetworkClientCommunicator};
use std::collections::HashMap;
use raft::{NodeWorker, NewDataRequest, ClientResponseStatus, ClientRequestHandler};
use std::sync::Arc;

pub struct CaseCluster<Cc : ClientRequestHandler> {
	pub initial_nodes: Vec<u64>,
	pub peer_communicator: InProcPeerCommunicator,
	pub node_workers: Vec<NodeWorker>,
	pub client_handlers: HashMap<u64, Cc>
}

#[derive(Clone)]
pub struct Leader {
	pub id : u64,
	pub client_handler: Arc<NetworkClientCommunicator>
}

pub fn start_initial_cluster(nodes : Vec<u64>, peer_communicator : InProcPeerCommunicator) -> CaseCluster<NetworkClientCommunicator> {
	let all_nodes = nodes.clone();

	let mut client_handlers  = HashMap::new();
	let mut node_workers = Vec::new();

	//run initial cluster
	for node_id in all_nodes.clone() {
		let (client_request_handler, node_config) = super::create_node_configuration(node_id, all_nodes.clone(),peer_communicator.clone() );

		let node_worker = raft::start_node(node_config);
		node_workers.push(node_worker);

		client_handlers.insert(node_id, client_request_handler);
	}

	CaseCluster{
		initial_nodes: nodes,
		peer_communicator,
		node_workers,
		client_handlers
	}
}

impl CaseCluster<NetworkClientCommunicator> {
	pub fn add_new_server(&mut self, new_node_id: u64) {
		let (client_request_handler, new_node_config) = super::create_node_configuration(new_node_id, self.initial_nodes.clone(), self.peer_communicator.clone());
		let node_worker = raft::start_node(new_node_config);
		self.node_workers.push(node_worker);
		self.client_handlers.insert(new_node_id, client_request_handler);
	}

	pub fn find_a_leader(&self) -> Leader{
		let bytes = "find a leader".as_bytes();
		let new_data_request = NewDataRequest{data : Arc::new(bytes)};
		for kv in &self.client_handlers {
			let (k, v) = kv;

			let result = v.new_data(new_data_request.clone());
			if let Ok(resp) = result {
				if let ClientResponseStatus::Ok = resp.status {
					let mut client_handler = Arc::new(v.clone());
					let leader_id = resp.current_leader.expect("can get a leader");

					if *k != leader_id {
						client_handler = Arc::new(self.client_handlers[&leader_id].clone());
					}
					return Leader {client_handler, id : leader_id};
				}
			}
		}

		panic!("cannot get a leader!")
	}

	pub fn terminate(self) {
		let mut handles = Vec::new();
		for node_worker in self.node_workers {
			let handle = node_worker.join_handle;
			handles.push(handle);
			let thread = node_worker.terminate_worker_tx.send(());
			if thread.is_err(){
				panic!("worker panicked!")
			}
		}

		for node_worker in handles {
			let thread = node_worker.join();
			if thread.is_err(){
				panic!("worker panicked!")
			}
		}
	}

}
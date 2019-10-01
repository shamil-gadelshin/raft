use std::collections::HashMap;
use raft::{NodeWorker, NewDataRequest, ClientRequestHandler, PeerRequestHandler, PeerRequestChannels};
use std::sync::Arc;

pub struct CaseCluster<Cc, Pc>
	where Cc : ClientRequestHandler,
		  Pc : PeerRequestHandler + PeerRequestChannels{
	pub initial_nodes: Vec<u64>,
	pub peer_communicator: Pc,
	pub node_workers: Vec<NodeWorker>,
	pub client_handlers: HashMap<u64, Cc>
}

#[derive(Clone)]
pub struct Leader<Cc : ClientRequestHandler> {
	pub id : u64,
	pub client_handler: Arc<Cc>
}



pub fn start_initial_cluster<F, Cc, Pc>(nodes : Vec<u64>, peer_communicator : Pc, node_creator: F) -> CaseCluster<Cc, Pc>
where F: Fn(u64, Vec<u64>, Pc) -> (NodeWorker, Cc),
	  Cc : ClientRequestHandler,
      Pc : PeerRequestHandler + PeerRequestChannels{
	let all_nodes = nodes.clone();

	let mut client_handlers  = HashMap::new();
	let mut node_workers = Vec::new();

	//run initial cluster
	for node_id in all_nodes.clone() {
		let (node_worker, client_request_handler) = node_creator(node_id, all_nodes.clone(), peer_communicator.clone());

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

impl <Cc, Pc> CaseCluster<Cc, Pc>
where Cc : ClientRequestHandler,
	  Pc : PeerRequestHandler + PeerRequestChannels {
	pub fn add_new_server<F>(&mut self, new_node_id: u64, node_creator: F)
		where F: Fn(u64, Vec<u64>, Pc)  -> (NodeWorker, Cc) {
		let (node_worker, client_request_handler) = node_creator(new_node_id, self.initial_nodes.clone(), self.peer_communicator.clone());

		self.node_workers.push(node_worker);
		self.client_handlers.insert(new_node_id, client_request_handler);
	}

	pub fn find_a_leader_by_adding_data_sample(&self) -> Leader<Cc>{
		let bytes = "find a leader".as_bytes();
		let new_data_request = NewDataRequest{data : Arc::new(bytes)};
		for kv in &self.client_handlers {
			let (_k, v) = kv;

			let result = v.new_data(new_data_request.clone());
			info!("--Leader Result: {:?}", result);
			if let Ok(resp) = result {
				let leader_id = resp.current_leader.expect("can get a leader");
				let	client_handler = Arc::new(self.client_handlers[&leader_id].clone());

				return Leader {client_handler, id : leader_id};
			}

			error!("Find a leader error: {:?}", result);
		}

		panic!("cannot get a leader!")
	}

	pub fn get_node1_leader_by_adding_data_sample(&self) -> Leader<Cc> {
		let bytes = "find a leader".as_bytes();
		let new_data_request = NewDataRequest { data: Arc::new(bytes) };


		let handler = self.client_handlers[&1].clone();

		let result = handler.new_data(new_data_request.clone());
		info!("--Leader Result: {:?}", result);
		if let Ok(resp) = result {
			let leader_id = resp.current_leader.expect("can get a leader");
			let client_handler = Arc::new(self.client_handlers[&leader_id].clone());

			return Leader { client_handler, id: leader_id };
		}

		error!("Find a leader error: {:?}", result);

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
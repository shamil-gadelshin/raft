use crate::steps;
use raft::{PeerRequestHandler, PeerRequestChannels, NodeConfiguration, ElectionTimer, NodeState, NodeTimings, OperationLog, DataEntryContent};
use raft_modules::{InProcClientCommunicator, MemoryRsm, RandomizedElectionTimer, MockNodeStateSaver, ClusterConfiguration};
use crate::steps::get_client_communication_timeout;

pub fn run() {

	let node_ids = vec![1, 2];
	let new_node_id = node_ids.last().unwrap() + 1;

	let peer_communicator = steps::peer_communicator::get_peer_communicator( vec![1, 2, 3]);
	let mut cluster = steps::cluster::start_initial_cluster(node_ids, peer_communicator.clone(), steps::create_node_inproc);

	steps::sleep(2);

	//find elected leader
	let leader = cluster.get_node1_leader_by_adding_data_sample();

	//add new data to the cluster
	steps::data::add_data_sample(&leader).expect("add sample successful");

	let (tx, rx): (Sender<Vec<LogEntry>>, Receiver<Vec<LogEntry>>) = crossbeam_channel::unbounded();
	let mut operation_log = MemoryOperationLog::new(ClusterConfiguration::new(cluster.initial_nodes.clone()), tx);
	operation_log.append_entry(
		LogEntry{
		term: 1, index: 1, entry_content: EntryContent::Data(
			DataEntryContent{
				data:Arc::new(b"bytes")
			})
		}
	).expect("append successful");
	// run new server
	cluster.add_new_server(new_node_id,  |node_id, all_nodes, peer_communicator| {
		let election_timer = RandomizedElectionTimer::new(2000, 4000);
		let (client_request_handler, node_config) = create_node_configuration_inproc(node_id, all_nodes, peer_communicator, election_timer, operation_log.clone());
		let node_worker = raft::start_node(node_config);

		(node_worker, client_request_handler)
	});

	//add new server to the cluster
	steps::data::add_server(&leader, new_node_id);

	steps::sleep(1);
	//add new data to the cluster
	steps::data::add_data_sample(&leader).expect("add sample successful");

	steps::sleep(5);

	let mut entry_count = 0;
	loop {
		let res = rx.try_recv();

		if let Ok(val) = res {
			entry_count+=1;
			println!("{:?}", val);
		} else {
			// println!("{:?}", res);
			break;
		}
	}

//	assert_eq!(4, entry_count);


	cluster.terminate();
}

fn create_node_configuration_inproc<Pc, Et, Log>(node_id: u64, all_nodes: Vec<u64>, communicator: Pc, election_timer : Et, operation_log: Log )
											-> (InProcClientCommunicator, NodeConfiguration<Log, MemoryRsm, InProcClientCommunicator, Pc, Et, MockNodeStateSaver, ClusterConfiguration>)
	where  Pc : PeerRequestHandler + PeerRequestChannels,
		   Et : ElectionTimer,
		   Log : OperationLog{
	let cluster_config =ClusterConfiguration::new(all_nodes);
	let client_request_handler = InProcClientCommunicator::new(node_id, get_client_communication_timeout());

	let config = NodeConfiguration {
		node_state: NodeState {
			node_id,
			current_term: 0,
			vote_for_id: None
		},
		cluster_configuration: cluster_config.clone(),
		peer_communicator: communicator,
		client_communicator: client_request_handler.clone(),
		election_timer,
		operation_log,
		rsm: MemoryRsm::default(),
		state_saver: MockNodeStateSaver::default(),
		timings: NodeTimings::default()
	};

	(client_request_handler, config)
}

use raft::{LogEntry, EntryContent};
use raft::{RaftError};
use crossbeam_channel::{Sender, Receiver};
use std::sync::Arc;

#[derive(Clone, Debug)]
//TODO multithreaded protection required
pub struct MemoryOperationLog {
	cluster: ClusterConfiguration,
	last_index: u64,
	entries : Vec<LogEntry>,
	debug_tx: Sender<Vec<LogEntry>>
}

impl MemoryOperationLog {
	pub fn new(cluster: ClusterConfiguration, debug_tx: Sender<Vec<LogEntry>>)-> MemoryOperationLog {
		MemoryOperationLog {
			cluster,
			entries : Vec::new(),
			last_index: 0,
			debug_tx
		}
	}

	fn add_servers_to_cluster(&mut self, servers: Vec<u64>) {
		for new_server_id in servers {
			self.cluster.add_peer(new_server_id);
		}
	}

}

impl OperationLog for MemoryOperationLog {
	fn create_next_entry(&mut self,  term : u64, entry_content : EntryContent) -> LogEntry {
		LogEntry { index: self.last_index + 1, term, entry_content }
	}

	fn append_entry(&mut self, entry: LogEntry) -> Result<(), RaftError> {
		while self.last_index >= entry.index {
			let vec_idx = (entry.index - 1) as usize;
			let removed_entry = self.entries.remove(vec_idx);
			self.last_index -= 1;
			warn!("-------------Removed Log entry: {:?}", removed_entry);
		}

		self.entries.push(entry.clone());
		self.debug_tx.send(self.entries.clone()).expect("valid send");

		if let EntryContent::AddServer(add_server_request) = entry.entry_content {
			trace!("New server configuration: {:?}", &add_server_request.new_cluster_configuration);
			self.add_servers_to_cluster(add_server_request.new_cluster_configuration);
		}

		self.last_index = entry.index;


		Ok(())
	}
	fn get_entry(&self, index: u64) -> Option<LogEntry> {
		let idx = index as usize;
		if idx > self.entries.len() || idx == 0{
			return None;
		}
		Some(self.entries[idx-1].clone())
	}

	fn get_last_entry_index(&self) -> u64{
		let last = self.entries.last();
		if let Some(entry) = last {
			return entry.index;
		}

		0 //Raft documentation demands zero as initial value
	}
	fn get_last_entry_term(&self) -> u64{
		let last = self.entries.last();
		if let Some(entry) = last {
			return entry.term;
		}

		0 //Raft documentation demands zero as initial value
	}
}


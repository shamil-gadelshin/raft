use crate::configuration::cluster::ClusterConfiguration;
use std::sync::{Mutex, Arc};
use crate::operation_log::storage::LogStorage;
use crate::common::{LogEntry, DataEntryContent, EntryContent};

//TODO crate fsm trait
#[derive(Debug, Clone)]
pub struct Fsm {
	cluster_configuration: Arc<Mutex<ClusterConfiguration>>,
	data : Vec<DataEntryContent>,
	last_applied_index: u64,
}

impl Fsm{
	pub fn new(cluster_configuration: Arc<Mutex<ClusterConfiguration>>)->Fsm {
		Fsm{
			cluster_configuration,
			data : Vec::new(),
			last_applied_index: 0
		}
	}
	pub fn apply_entry(&mut self, entry: LogEntry) {
		match entry.entry_content {
			EntryContent::AddServer(add_server_content) => {
				self.add_server_to_cluster(add_server_content.new_server);
			},
			EntryContent::Data(data_content) => {
				self.data.push(data_content);
			}
		}
		self.last_applied_index = entry.index;
	}

	fn add_server_to_cluster(&self, new_server_id: u64) {
		let mut cluster = self.cluster_configuration.lock().expect("cluster lock is not poisoned");

		cluster.add_peer(new_server_id);
	}

	pub fn get_last_entry_index(&self) -> u64{
		self.last_applied_index
	}
}
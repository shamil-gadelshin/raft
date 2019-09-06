use std::sync::{Mutex, Arc};
use std::error::Error;

use ruft::ClusterConfiguration;
use ruft::{LogEntry, DataEntryContent, EntryContent, Fsm};

#[derive(Debug, Clone)]
pub struct MemoryFsm {
	cluster_configuration: Arc<Mutex<ClusterConfiguration>>,
	data : Vec<DataEntryContent>,
	last_applied_index: u64,
}

impl MemoryFsm {
	pub fn new(cluster_configuration: Arc<Mutex<ClusterConfiguration>>)-> MemoryFsm {
		MemoryFsm {
			cluster_configuration,
			data : Vec::new(),
			last_applied_index: 0,
		}
	}

	fn add_server_to_cluster(&self, new_server_id: u64) {
		let mut cluster = self.cluster_configuration.lock().expect("cluster lock is not poisoned");

		cluster.add_peer(new_server_id);
	}

}

impl Fsm for MemoryFsm {
	fn apply_entry(&mut self, entry: LogEntry) -> Result<(), Box<Error>> {
		if self.get_last_applied_entry_index() >= entry.index {
			warn!("Attempted to apply entry with existing index={}", entry.index);
			return Ok(())
		}

		match entry.entry_content {
			EntryContent::AddServer(add_server_content) => {
				self.add_server_to_cluster(add_server_content.new_server);
			},
			EntryContent::Data(data_content) => {
				self.data.push(data_content);
			}
		}
		trace!("Fsm applied entry: {}", entry.index);
		self.last_applied_index = entry.index;

		Ok(())
	}


	fn get_last_applied_entry_index(&self) -> u64{
		self.last_applied_index
	}
}
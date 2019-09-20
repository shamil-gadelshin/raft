use raft::{LogEntry, DataEntryContent, EntryContent, ReplicatedStateMachine};
use raft::{RaftError};

#[derive(Debug, Clone, Default)]
//TODO multithreaded protection required
pub struct MemoryRsm {
	data : Vec<DataEntryContent>,
	last_applied_index: u64,
}


impl ReplicatedStateMachine for MemoryRsm {
	fn apply_entry(&mut self, entry: LogEntry) -> Result<(), RaftError> {
		if self.get_last_applied_entry_index() >= entry.index {
			warn!("Attempted to apply entry with existing index={}", entry.index);
			return Ok(())
		}

		if let EntryContent::Data(data_content) = entry.entry_content {
			self.data.push(data_content);
		}

		trace!("Rsm applied entry: {}", entry.index);
		self.last_applied_index = entry.index;

		Ok(())
	}


	fn get_last_applied_entry_index(&self) -> u64{
		self.last_applied_index
	}
}
use std::error::Error;

use raft::{LogEntry, DataEntryContent, EntryContent, FiniteStateMachine};

#[derive(Debug, Clone, Default)]
pub struct MemoryFsm {
	data : Vec<DataEntryContent>,
	last_applied_index: u64,
}


impl FiniteStateMachine for MemoryFsm {
	fn apply_entry(&mut self, entry: LogEntry) -> Result<(), Box<Error>> {
		if self.get_last_applied_entry_index() >= entry.index {
			warn!("Attempted to apply entry with existing index={}", entry.index);
			return Ok(())
		}

		if let EntryContent::Data(data_content) = entry.entry_content {
			self.data.push(data_content);
		}

		trace!("Fsm applied entry: {}", entry.index);
		self.last_applied_index = entry.index;

		Ok(())
	}


	fn get_last_applied_entry_index(&self) -> u64{
		self.last_applied_index
	}
}
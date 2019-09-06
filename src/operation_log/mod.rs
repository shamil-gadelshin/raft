pub mod replication;

use crate::common::{LogEntry, EntryContent};

pub trait LogStorage {
	fn create_next_entry(&mut self, term : u64, entry_content : EntryContent)-> LogEntry; //TODO error handling
	fn append_entry(&mut self, entry: LogEntry); //TODO error handling
	fn get_entry(&self, index : u64) -> Option<LogEntry>;
	fn get_last_entry_index(&self) -> u64;
	fn get_last_entry_term(&self) -> u64;
}

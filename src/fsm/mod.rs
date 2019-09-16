pub mod updater;

use std::error::Error;

use crate::common::{LogEntry};

pub trait FiniteStateMachine: Sized + Sync + Send +  'static {
	fn apply_entry(&mut self, entry: LogEntry) -> Result<(), Box<Error>>;
	fn get_last_applied_entry_index(&self) -> u64;
}

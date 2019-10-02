pub mod updater;

use crate::errors::RaftError;
use crate::operation_log::LogEntry;

pub trait ReplicatedStateMachine: Sync + Send + 'static {
    fn apply_entry(&mut self, entry: LogEntry) -> Result<(), RaftError>;
    fn get_last_applied_entry_index(&self) -> u64;
}

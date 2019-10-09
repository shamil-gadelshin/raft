pub mod updater;

use crate::errors::RaftError;
use crate::operation_log::LogEntry;

/// Provides Raft operations with the underlying replicated state machine.
pub trait ReplicatedStateMachine: Sync + Send + 'static {
    /// Apply operation log entry to the state machine.
    fn apply_entry(&mut self, entry: LogEntry) -> Result<(), RaftError>;

    /// Returns index of the last operation log entry applied to the state machine.
    fn last_applied_entry_index(&self) -> u64;
}

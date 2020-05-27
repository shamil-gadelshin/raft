pub mod replication;

use crate::errors::RaftError;
use std::sync::Arc;

pub trait QuorumResponse: Send {
    fn result(&self) -> bool;
}

/// Data container for the operation log entry. Byte slice wrapper.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Display)]
#[display(fmt = "Data size: {}", "data.len()")]
pub struct DataEntryContent {
    /// New data (command) serialized in bytes format.
    pub data: Arc<&'static [u8]>,
}

/// Contains new cluster configuration. Vector of the node's id.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Default, Display)]
#[display(fmt = "New cluster configuration: {:?}", new_cluster_configuration)]
pub struct NewClusterConfigurationEntryContent {
    /// New cluster configuration after the adding new server tho the cluster.
    pub new_cluster_configuration: Vec<u64>,
}

/// Main abstraction for the Raft operation log entries.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Display)]
#[display(
    fmt = "Log entry: index {} term {} content {}",
    index,
    term,
    entry_content
)]
pub struct LogEntry {
    /// Raft operation log entry index. Increases monotonically with no gaps.
    pub index: u64,

    /// Raft operation log entry term. It is Raft's logical clock.
    pub term: u64,

    /// Raft operation log entry content. Actual content depends on entry type.
    pub entry_content: EntryContent,
}

/// Defines content type for operation log entry.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Display)]
pub enum EntryContent {
    /// Provides info for adding server for the cluster.
    AddServer(NewClusterConfigurationEntryContent),
    /// Contains new data.
    Data(DataEntryContent),
}

/// Abstraction of the system operation log.
pub trait OperationLog: Sync + Send + 'static {
    /// Create new entry with given term and content. New entry contains calculated last index.
    fn create_next_entry(&mut self, term: u64, entry_content: EntryContent) -> LogEntry;

    /// Appends entry to the operation log.
    fn append_entry(&mut self, entry: LogEntry) -> Result<(), RaftError>;

    /// Returns operation log entry by index.
    fn entry(&self, index: u64) -> Option<LogEntry>;

    /// Returns last operation log entry index.
    fn last_entry_index(&self) -> u64;

    /// Returns last operation log entry term.
    fn last_entry_term(&self) -> u64;
}

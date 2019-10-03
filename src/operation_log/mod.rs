pub mod replication;

use crate::errors::RaftError;
use std::sync::Arc;

pub trait QuorumResponse: Send {
    fn get_result(&self) -> bool;
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Display)]
#[display(fmt = "Data size: {}", "data.len()")]
pub struct DataEntryContent {
    pub data: Arc<&'static [u8]>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default, Display)]
#[display(fmt = "New cluster configuration: {:?}", new_cluster_configuration)]
pub struct NewClusterConfigurationEntryContent {
    pub new_cluster_configuration: Vec<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Display)]
#[display(fmt = "Log entry: index {} term {} content {}", index, term, entry_content)]
pub struct LogEntry {
    pub index: u64,
    pub term: u64,
    pub entry_content: EntryContent,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Display)]
pub enum EntryContent {
    AddServer(NewClusterConfigurationEntryContent),
    Data(DataEntryContent),
}

pub trait OperationLog: Sync + Send + 'static {
    fn create_next_entry(&mut self, term: u64, entry_content: EntryContent) -> LogEntry;
    fn append_entry(&mut self, entry: LogEntry) -> Result<(), RaftError>;
    fn get_entry(&self, index: u64) -> Option<LogEntry>;
    fn get_last_entry_index(&self) -> u64;
    fn get_last_entry_term(&self) -> u64;
}

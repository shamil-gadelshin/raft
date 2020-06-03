use crate::ClusterConfiguration;
use parking_lot::Mutex;
use raft::OperationLog;
use raft::RaftError;
use raft::{EntryContent, LogEntry};
use std::sync::Arc;

/// Basic in-memory implementation of the OperationLog trait. It is responsible for updating
/// the cluster configuration.
#[derive(Clone, Debug)]
pub struct MemoryOperationLog {
    cluster: ClusterConfiguration,
    last_index: u64,
    entries: Vec<LogEntry>,
    lock: Arc<Mutex<()>>,
}

impl MemoryOperationLog {
    /// Creates new MemoryOperationLog initialized with cluster configuration.
    pub fn new(cluster: ClusterConfiguration) -> MemoryOperationLog {
        MemoryOperationLog {
            cluster,
            entries: Vec::new(),
            last_index: 0,
            lock: Arc::new(Mutex::new(())),
        }
    }
}

impl OperationLog for MemoryOperationLog {
    fn create_next_entry(&mut self, term: u64, entry_content: EntryContent) -> LogEntry {
        let _lock = self.lock.lock();
        LogEntry {
            index: self.last_index + 1,
            term,
            entry_content,
        }
    }

    fn append_entry(&mut self, entry: LogEntry) -> Result<(), RaftError> {
        let _lock = self.lock.lock();

        trace!("--Log entry: {}", entry);
        while self.last_index >= entry.index {
            let vec_idx = (entry.index - 1) as usize;
            let removed_entry = self.entries.remove(vec_idx);
            self.last_index -= 1;
            warn!("-------------Removed Log entry: {}", removed_entry);
        }

        self.entries.push(entry.clone());

        if let EntryContent::AddServer(add_server_request) = entry.entry_content {
            trace!(
                "New server configuration: {:?}",
                &add_server_request.new_cluster_configuration
            );

            // add servers to cluster
            for new_server_id in add_server_request.new_cluster_configuration {
                self.cluster.add_peer(new_server_id);
            }
        }

        self.last_index = entry.index;

        Ok(())
    }

    fn entry(&self, index: u64) -> Option<LogEntry> {
        let _lock = self.lock.lock();

        let idx = index as usize;
        if idx > self.entries.len() || idx == 0 {
            return None;
        }
        Some(self.entries[idx - 1].clone())
    }

    fn last_entry_index(&self) -> u64 {
        let _lock = self.lock.lock();
        let last = self.entries.last();
        if let Some(entry) = last {
            return entry.index;
        }

        0 //Raft documentation demands zero as initial value
    }
    fn last_entry_term(&self) -> u64 {
        let _lock = self.lock.lock();
        let last = self.entries.last();
        if let Some(entry) = last {
            return entry.term;
        }

        0 //Raft documentation demands zero as initial value
    }
}

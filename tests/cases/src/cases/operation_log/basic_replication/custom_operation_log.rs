use crossbeam_channel::Sender;
use raft::RaftError;
use raft::{EntryContent, LogEntry, OperationLog};
use raft_modules::ClusterConfiguration;

#[derive(Clone, Debug)]
pub struct MemoryOperationLog {
    cluster: ClusterConfiguration,
    last_index: u64,
    entries: Vec<LogEntry>,
    debug_tx: Sender<LogEntry>,
}

impl MemoryOperationLog {
    pub fn new(cluster: ClusterConfiguration, debug_tx: Sender<LogEntry>) -> MemoryOperationLog {
        MemoryOperationLog {
            cluster,
            entries: Vec::new(),
            last_index: 0,
            debug_tx,
        }
    }

    fn add_servers_to_cluster(&mut self, servers: Vec<u64>) {
        for new_server_id in servers {
            self.cluster.add_peer(new_server_id);
        }
    }
}

impl OperationLog for MemoryOperationLog {
    fn create_next_entry(&mut self, term: u64, entry_content: EntryContent) -> LogEntry {
        LogEntry {
            index: self.last_index + 1,
            term,
            entry_content,
        }
    }

    fn append_entry(&mut self, entry: LogEntry) -> Result<(), RaftError> {
        if self.last_index < entry.index {
            self.entries.push(entry.clone());
            self.debug_tx.send(entry.clone()).expect("valid send");

            if let EntryContent::AddServer(add_server_request) = entry.entry_content {
                trace!(
                    "New server configuration: {:?}",
                    &add_server_request.new_cluster_configuration
                );
                self.add_servers_to_cluster(add_server_request.new_cluster_configuration);
            }

            self.last_index = entry.index;
        }

        Ok(())
    }
    fn entry(&self, index: u64) -> Option<LogEntry> {
        let idx = index as usize;
        if idx > self.entries.len() || idx == 0 {
            return None;
        }
        Some(self.entries[idx - 1].clone())
    }

    fn last_entry_index(&self) -> u64 {
        let last = self.entries.last();
        if let Some(entry) = last {
            return entry.index;
        }

        0 //Raft documentation demands zero as initial value
    }
    fn last_entry_term(&self) -> u64 {
        let last = self.entries.last();
        if let Some(entry) = last {
            return entry.term;
        }

        0 //Raft documentation demands zero as initial value
    }
}

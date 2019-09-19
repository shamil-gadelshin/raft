use raft::{LogEntry, EntryContent, ClusterConfiguration};
use raft::OperationLog;
use std::error::Error;
use std::sync::{Mutex, Arc};

#[derive(Clone, Debug)]
pub struct MemoryOperationLog {
    cluster_configuration: Arc<Mutex<ClusterConfiguration>>,
    last_index: u64,
    entries : Vec<LogEntry>
}

impl MemoryOperationLog {
    pub fn new(cluster_configuration: Arc<Mutex<ClusterConfiguration>>)-> MemoryOperationLog {
        MemoryOperationLog {
            cluster_configuration,
            entries : Vec::new(),
            last_index: 0,
        }
    }

    fn add_servers_to_cluster(&self, servers: Vec<u64>) {
        let mut cluster = self.cluster_configuration.lock().expect("cluster lock is not poisoned");

        for new_server_id in servers {
            cluster.add_peer(new_server_id);
        }
    }
}

impl OperationLog for MemoryOperationLog {
    fn create_next_entry(&mut self,  term : u64, entry_content : EntryContent) -> LogEntry {
        LogEntry { index: self.last_index + 1, term, entry_content }
    }

    fn append_entry(&mut self, entry: LogEntry) -> Result<(), Box<Error>> {
        if self.last_index < entry.index {
            self.entries.push(entry.clone());

            if let EntryContent::AddServer(add_server_request) =  entry.entry_content {
                trace!("New server configuration: {:?}", &add_server_request.new_cluster_configuration);
                self.add_servers_to_cluster(add_server_request.new_cluster_configuration);
            }

            self.last_index = entry.index;
        }

        Ok(())
    }
    fn get_entry(&self, index: u64) -> Option<LogEntry> {
        let idx = index as usize;
        if idx > self.entries.len() {
            return None;
        }
        Some(self.entries[idx-1].clone())
    }

    fn get_last_entry_index(&self) -> u64{
        let last = self.entries.last();
        if let Some(entry) = last {
            return entry.index;
        }

        0 //Raft documentation demands zero as initial value
    }
    fn get_last_entry_term(&self) -> u64{
        let last = self.entries.last();
        if let Some(entry) = last {
            return entry.term;
        }

        0 //Raft documentation demands zero as initial value
    }
}




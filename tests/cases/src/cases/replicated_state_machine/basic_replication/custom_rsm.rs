use crossbeam_channel::Sender;
use raft::RaftError;
use raft::{DataEntryContent, EntryContent, LogEntry, ReplicatedStateMachine};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct MemoryRsm {
    data: Vec<DataEntryContent>,
    last_applied_index: u64,
    lock: Arc<Mutex<bool>>,
    debug_tx: Sender<DataEntryContent>,
}

impl MemoryRsm {
    pub fn new(debug_tx: Sender<DataEntryContent>) -> MemoryRsm {
        MemoryRsm {
            data: Vec::new(),
            last_applied_index: 0,
            lock: Arc::new(Mutex::new(true)),
            debug_tx,
        }
    }
}

impl ReplicatedStateMachine for MemoryRsm {
    fn apply_entry(&mut self, entry: LogEntry) -> Result<(), RaftError> {
        let _lock = self.lock.lock();

        if self.last_applied_index >= entry.index {
            warn!(
                "Attempted to apply entry with existing index={}",
                entry.index
            );
            return Ok(());
        }

        if let EntryContent::Data(data_content) = entry.entry_content {
            self.data.push(data_content.clone());
            self.debug_tx
                .send(data_content.clone())
                .expect("valid send");
        }

        trace!("Rsm applied entry: {}", entry.index);
        self.last_applied_index = entry.index;

        Ok(())
    }

    fn last_applied_entry_index(&self) -> u64 {
        let _lock = self.lock.lock();

        self.last_applied_index
    }
}

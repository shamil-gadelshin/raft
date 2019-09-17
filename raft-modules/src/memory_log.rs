use raft::{LogEntry, EntryContent};
use raft::OperationLog;
use std::error::Error;

#[derive(Clone, Debug, Default)]
pub struct MemoryLogStorage {
    last_index: u64,
    entries : Vec<LogEntry>
}

impl OperationLog for MemoryLogStorage {
    fn create_next_entry(&mut self,  term : u64, entry_content : EntryContent) -> LogEntry {
        LogEntry { index: self.last_index + 1, term, entry_content }
    }

    fn append_entry(&mut self, entry: LogEntry) -> Result<(), Box<Error>> {
        if self.last_index < entry.index {
            self.last_index = entry.index;
            self.entries.push(entry);
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




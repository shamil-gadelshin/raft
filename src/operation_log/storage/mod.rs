use crate::common::{LogEntry, EntryContent};

pub trait LogStorage {
    fn append_content(&mut self, term : u64, entry_content : EntryContent)-> LogEntry; //TODO error handling
    fn append_entry(&mut self, entry: LogEntry); //TODO error handling
    fn get_last_entry_index(&self) -> usize;
    fn get_last_entry_term(&self) -> u64;
    fn get_entry(&self, index : usize) -> Option<LogEntry>;
}

#[derive(Clone, Debug)]
pub struct MemoryLogStorage {
    last_index: usize,
    entries : Vec<LogEntry>
}

impl LogStorage for MemoryLogStorage {
    //TODO check for duplicates
    fn append_entry(&mut self, entry: LogEntry) {
        self.last_index+=1;
        self.entries.push(entry);
    }
    fn append_content(&mut self,  term : u64, entry_content : EntryContent) -> LogEntry {
        let entry = LogEntry { index: self.last_index, term, entry_content };
        self.append_entry(entry.clone());

        entry
    }
    fn get_last_entry_index(&self) -> usize{
        let last = self.entries.last();
        if let Some(entry) = last {
            return entry.index;
        }

        0 //Raft documentation demands zero as initial value
    }
    fn get_last_entry_term(&self) -> u64 {
        let last = self.entries.last();
        if let Some(entry) = last {
            return entry.term;
        }

        0 //Raft documentation demands zero as initial value
    }

    fn get_entry(&self, index: usize) -> Option<LogEntry> {
        if index >= self.entries.len() {
            return None;
        }
        Some(self.entries[index].clone())
    }
}

impl MemoryLogStorage {
    pub fn new() -> MemoryLogStorage {
        MemoryLogStorage {last_index:0, entries : Vec::new()}
    }
}




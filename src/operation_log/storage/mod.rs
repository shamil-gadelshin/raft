use crate::common::{LogEntry, EntryContent};

pub trait LogStorage {
    fn append_content(&mut self, term : u64, entry_content : EntryContent)-> LogEntry; //TODO error handling
    fn append_entry(&mut self, entry: LogEntry); //TODO error handling
    fn get_entry(&self, index : usize) -> Option<LogEntry>;
    fn get_last_entry_index(&self) -> usize;
    fn get_last_entry_term(&self) -> u64;
}

#[derive(Clone, Debug)]
pub struct MemoryLogStorage {
    last_index: usize, //TODO remove?
    entries : Vec<LogEntry>
}

impl LogStorage for MemoryLogStorage {
    fn append_content(&mut self,  term : u64, entry_content : EntryContent) -> LogEntry {
        let entry = LogEntry { index: self.last_index + 1, term, entry_content };
        self.append_entry(entry.clone());
        entry
    }

    //TODO check for duplicates
    fn append_entry(&mut self, entry: LogEntry) {
        self.last_index = entry.index;
        self.entries.push(entry);
    }
    fn get_entry(&self, index: usize) -> Option<LogEntry> {
        if index > self.entries.len() {
            return None;
        }
        Some(self.entries[index-1].clone())
    }

    //TODO simplify to self.last_index?
    fn get_last_entry_index(&self) -> usize{
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

impl MemoryLogStorage {
    pub fn new() -> MemoryLogStorage {
        MemoryLogStorage {last_index:0, entries : Vec::new()}
    }
}




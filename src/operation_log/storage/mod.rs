use crate::common::{AddServerEntryContent, DataEntryContent};

pub trait LogStorage {
    fn append_entry(&mut self, term : u64, entry_content : EntryContent); //TODO error handling
    fn get_last_entry_index(&self) -> u64;
    fn get_last_entry_term(&self) -> u64;
}

#[derive(Clone, Debug)]
pub struct MemoryLogStorage {
    last_index: u64,
    entries : Vec<Entry>
}

impl LogStorage for MemoryLogStorage {
    //TODO check for duplicates
    fn append_entry(&mut self, term : u64, entry_content : EntryContent) {
        self.last_index+=1;
        let entry = Entry{index : self.last_index, term, entry_content };
        self.entries.push(entry);
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

impl MemoryLogStorage {
    pub fn new() -> MemoryLogStorage {
        MemoryLogStorage {last_index:0, entries : Vec::new()}
    }
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub index: u64,
    pub term: u64,
    pub entry_content: EntryContent
}

#[derive(Clone, Debug)]
pub enum EntryContent {
    AddServer(AddServerEntryContent),
    Data(DataEntryContent),
}



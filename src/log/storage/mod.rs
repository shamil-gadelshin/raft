use crate::common::{AddServerEntryDetails, DataEntryDetails};

pub trait LogStorage {
    fn append_entry(&mut self, entry : Entry);
    fn get_last_entry_index(&self) -> u64;
    fn get_last_entry_term(&self) -> u64;
}

#[derive(Clone, Debug)]
pub struct MemoryLogStorage {
    entries : Vec<Entry>
}

impl LogStorage for MemoryLogStorage {
    //TODO check for duplicates
    fn append_entry(&mut self, entry : Entry) {
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
        MemoryLogStorage {entries : Vec::new()}
    }
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub index: u64,
    pub term: u64,
    pub entry_type : EntryType
}

#[derive(Clone, Debug)]
pub enum EntryType {
    AddServer(AddServerEntryDetails),
    Data(DataEntryDetails),
}



use std::rc::Rc;
use std::collections::HashMap;
use std::sync::Arc;

pub trait Storage {
    fn append_entry(&mut self, entry : Entry);
}

pub struct MemoryStorage {
    entries : HashMap<u64, Entry>
}

impl Storage for MemoryStorage{
    //TODO check duplicates (success)
    fn append_entry(&mut self, entry : Entry) {
        self.entries.insert(entry.index, entry);
    }
}

impl MemoryStorage {
    pub fn new() -> MemoryStorage{
        MemoryStorage{entries : HashMap::new()}
    }
}

pub struct Entry {
    pub term_id : u64,
    pub index: u64,
    pub entry_type : EntryType
}

pub struct DataEntryDetails {
    pub bytes : Arc<[u8]> //TODO optimize - to Rc
}
pub struct AddServerEntryDetails {
    pub new_server : u64
}

pub enum EntryType {
    AddServer(AddServerEntryDetails),
    Data(DataEntryDetails),
}

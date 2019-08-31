use std::sync::{Arc};

pub enum LeaderConfirmationEvent {
    ResetWatchdogCounter
}

//TODO separate log & append entry implementations
#[derive(Clone, Debug)]
pub struct DataEntryContent {
    pub data : Arc<&'static [u8]>
}

//TODO separate log & append entry implementations
#[derive(Clone, Debug)]
pub struct AddServerEntryContent {
    pub new_server : u64
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub index: usize,
    pub term: u64,
    pub entry_content: EntryContent
}

#[derive(Clone, Debug)]
pub enum EntryContent {
    AddServer(AddServerEntryContent),
    Data(DataEntryContent),
}

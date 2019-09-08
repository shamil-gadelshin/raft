use std::sync::{Arc};
use std::thread::JoinHandle;
use std::thread;

//TODO downgrade to bool
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
    pub index: u64,
    pub term: u64,
    pub entry_content: EntryContent
}

#[derive(Clone, Debug)]
pub enum EntryContent {
    AddServer(AddServerEntryContent),
    Data(DataEntryContent),
}

pub fn run_worker_thread<T: Send + 'static, F: Fn(T) + Send + 'static>(worker : F, params : T) -> JoinHandle<()> {
    thread::spawn(move|| worker (params))
}
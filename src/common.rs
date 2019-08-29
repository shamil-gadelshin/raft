use std::sync::{Arc};

pub enum LeaderConfirmationEvent {
    ResetWatchdogCounter
}

//TODO separate log & append entry implementations
#[derive(Clone, Debug)]
pub struct DataEntryContent {
    pub bytes : Arc<[u8]>//TODO optimize with Rc
}

//TODO separate log & append entry implementations
#[derive(Clone, Debug)]
pub struct AddServerEntryContent {
    pub new_server : u64
}

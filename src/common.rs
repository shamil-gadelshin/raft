use std::sync::{Arc};

pub enum LeaderConfirmationEvent {
    ResetWatchdogCounter
}

#[derive(Clone, Debug)]
pub struct DataEntryDetails {
    pub bytes : Arc<[u8]>//TODO optimize with Rc
}

#[derive(Clone, Debug)]
pub struct AddServerEntryDetails {
    pub new_server : u64
}

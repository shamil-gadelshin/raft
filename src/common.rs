use chrono::prelude::{DateTime, Local};
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

pub fn print_event(message : String){
    let now: DateTime<Local> = Local::now();

    println!("{:?} {:?}",now.format("%H:%M:%S.%3f").to_string(),message );
}
use chrono::prelude::*;
use std::sync::{Arc};

//TODO ?
//pub trait Election {
//    fn request_vote(&self, request : communication::VoteRequest);
//    fn poll_for_vote_response(&self) -> VoteResponse;
//
//    fn send_vote(&self, response : VoteResponse);
//    fn poll_for_vote_requests(&self) -> VoteRequest;
//
////   fn send_leader_heartbeat();
//}



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
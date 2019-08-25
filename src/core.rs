use chrono::prelude::*;
use crate::log::storage::LogStorage;
use std::sync::{Arc};

#[derive(Debug, Clone)]
pub struct Node<Log: LogStorage + Sized + Sync> {
    pub id : u64, //TODO pass node_id as copy to decrease mutex lock count
    pub current_term: u64,
    pub current_leader_id: Option<u64>,
    pub voted_for_id: Option<u64>,
    pub status : NodeStatus,
    pub log : Log,
}

impl <Log: Sized + Sync + LogStorage> Node<Log> {
    pub fn get_last_entry_index(&self) -> u64{
       self.log.get_last_entry_index()
    }
    pub fn get_last_entry_term(&self) -> u64{
       self.log.get_last_entry_term()
    }
}

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


#[derive(Copy, Clone, Debug)]
pub enum NodeStatus {
    Follower,
    Candidate,
    Leader
}


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
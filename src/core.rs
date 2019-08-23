use chrono::prelude::*;
use std::collections::HashMap;

#[derive(Debug,Copy, Clone)]
pub struct Node {
    pub id : u64, //TODO pass node_id as copy to decrease mutex lock count
    pub current_term: u64,
    pub current_leader_id: Option<u64>, //TODO delete?
    pub voted_for_id: Option<u64>,
    pub status : NodeStatus
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

#[derive(Clone, Copy, Debug)]
pub struct VoteRequest {
    pub term : u64,
    pub candidate_id : u64
}

#[derive(Clone, Copy, Debug)]
pub struct AppendEntriesRequest {
    pub term : u64,
    pub leader_id : u64
}

#[derive(Clone, Copy, Debug)]
pub enum ChangeMembershipResponseStatus {
    Ok,
    NotLeader
}

#[derive(Clone, Copy, Debug)]
pub struct AddServerRequest {
    pub new_server : u64
}

#[derive(Clone, Copy, Debug)]
pub struct AddServerResponse {
    pub status : ChangeMembershipResponseStatus,
    pub current_leader : Option<u64>
}

#[derive(Clone, Copy, Debug)]
pub struct VoteResponse {
    pub term : u64,
    pub vote_granted: bool,
    pub peer_id: u64
}

pub fn print_event(message : String){

    let now: DateTime<Local> = Local::now();

    println!("{:?} {:?}",now.format("%H:%M:%S.%3f").to_string(),message );
}
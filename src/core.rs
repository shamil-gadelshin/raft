use chrono::prelude::*;

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
pub struct AppendEntriesRequest {
    pub term : u64,
    pub leader_id : u64
}

pub fn print_event(message : String){

    let now: DateTime<Local> = Local::now();

    println!("{:?} {:?}",now.format("%H:%M:%S.%3f").to_string(),message );
}
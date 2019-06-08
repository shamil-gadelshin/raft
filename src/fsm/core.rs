use std::time::{Duration};
use rand::Rng;
use chrono::prelude::*;

#[derive(Debug,Copy, Clone)]
pub struct Node {
    pub id : u64,
    pub current_term: u64,
    pub current_leader_id: Option<u64>, //TODO delete?
    pub voted_for_id: Option<u64>,
    pub status : NodeStatus
}

pub enum LeaderElectionEvent {
    PromoteNodeToCandidate(ElectionNotice),
    PromoteNodeToLeader(u64),
    ResetNodeToFollower(ElectionNotice),
}

pub enum LeaderElectedEvent {
    ResetWatchdogCounter
}


pub struct ElectionNotice {
    pub term : u64,
    pub candidate_id : u64
}

#[derive(Copy, Clone, Debug)]
pub enum NodeStatus {
    Follower,
    Candidate,
    Leader
}


pub fn random_awaiting_leader_duration_ms() -> Duration{
    let range_start = 1000;
    let range_stop = 4000;
    let mut rng = rand::thread_rng();

    Duration::from_millis(rng.gen_range(range_start, range_stop))
}

pub fn print_event(message : String){

    let now: DateTime<Local> = Local::now();

    println!("{:?} {:?}",now.format("%H:%M:%S.%3f").to_string(),message );
}
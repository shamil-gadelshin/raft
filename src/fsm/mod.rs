use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread::sleep;
use std::thread;
use std::collections::HashMap;
use rand::Rng;
use crossbeam_channel::{Sender, Receiver};

pub mod communication;

use self::communication::{VoteResponse, VoteRequest};
use self::communication::{InProcNodeCommunicator};
//
//impl Election for InProcNodeCommunicator {
//    fn request_vote(&self, request : VoteRequest) {
//        unimplemented!()
//    }
//
//    fn poll_for_vote_response(&self) -> VoteResponse {
//        unimplemented!()
//    }
//
//    fn send_vote(&self, response: VoteResponse) {
//        unimplemented!()
//    }
//
//    fn poll_for_vote_requests(&self) -> VoteRequest {
//        unimplemented!()
//    }
//}

pub trait Election {
    fn request_vote(&self, request : communication::VoteRequest);
    fn poll_for_vote_response(&self) -> VoteResponse;

    fn send_vote(&self, response : VoteResponse);
    fn poll_for_vote_requests(&self) -> VoteRequest;

//   fn send_leader_heartbeat();
}



pub struct NodeConfiguration {
    pub node_id: u64,
    pub peers_id_list : Vec<u64>,
    pub quorum_size: u32,
    pub request_rx_channel : Receiver<VoteRequest>,
    pub response_rx_channel : Receiver<VoteResponse>,
    pub communicator : InProcNodeCommunicator
}



#[derive(Copy, Clone, Debug)]
enum NodeStatus {
    Follower,
    Candidate,
    Leader
}

enum LeaderElectionEvent {
    PromoteNodeToCandidate(ElectionNotice),
    PromoteNodeToLeader(u64),
    SetNodeToFollower(ElectionNotice),
}

struct ElectionNotice {
    pub term : u64,
    pub candidate_id : u64
}

#[derive(Debug)]
struct Node {
    id : u64,
    current_term: u64,
    current_leader_id: Option<u64>,
    status : NodeStatus
}

fn debug_node_status(mutex_node: Arc<Mutex<Node>>) {
    loop {
        let node = mutex_node.lock().expect("lock is poisoned");
        println!("{:?}", node);

        sleep(random_awaiting_leader_duration_ms());
    }
}

fn watch_leader_status(mutex_node: Arc<Mutex<Node>>, event_tx : Sender<LeaderElectionEvent>) {
    loop {
        let timeout = crossbeam_channel::after(random_awaiting_leader_duration_ms());
        select!(
            recv(timeout) -> msg => {},
            //TODO : notify leadership
        );

        let node = mutex_node.lock().expect("lock is poisoned");

        if let NodeStatus::Follower = node.status {
            println!("Leader awaiting time elapsed. Starting new election.");

            let current_leader_id = node.current_leader_id.clone();

            if current_leader_id.is_none() || current_leader_id.unwrap() != node.id {
                let next_term = node.current_term + 1;
                let candidate_promotion = LeaderElectionEvent::PromoteNodeToCandidate(ElectionNotice { term: next_term, candidate_id: node.id });

                event_tx.send(candidate_promotion).expect("cannot promote to candidate");
            }
        }
    }
}

fn random_awaiting_leader_duration_ms() -> Duration{
    let range_start = 1500;
    let range_stop = 3000;
    let mut rng = rand::thread_rng();

    Duration::from_millis(rng.gen_range(range_start, range_stop))
}


fn run_leader_election_process(mutex_node: Arc<Mutex<Node>>,
                               event_tx : Sender<LeaderElectionEvent>,
                               event_rx : Receiver<LeaderElectionEvent>,
                               response_event_rx : Receiver<VoteResponse>,
                               communicator : InProcNodeCommunicator,
                               peers : Vec<u64>,
                               quorum_size : u32
                                ) {

    {
        let node = mutex_node.lock().expect("lock is poisoned");

        println!("Node {:?} started", node.id );
    }// mutex lock release

    loop {
        let event_result = event_rx.recv();

        let event = event_result.expect("receiving from closed channel");

        match event {
            LeaderElectionEvent::PromoteNodeToCandidate(vr) => {
                let mut node = mutex_node.lock().expect("lock is poisoned");

                node.status = NodeStatus::Candidate;
                node.current_leader_id = None;
                let event_sender = event_tx.clone();

                let node_id = node.id;
                let peers_copy = peers.clone();
                let communicator_copy = communicator.clone();
                let response_rx_copy = response_event_rx.clone();
                thread::spawn(move || notify_peers(vr.term,
                                                   event_sender,
                                                   node_id,
                                                   communicator_copy,
                                                   response_rx_copy,
                                                   peers_copy,
                                                   quorum_size));
            },
            LeaderElectionEvent::PromoteNodeToLeader(term) => {
                let mut node = mutex_node.lock().expect("lock is poisoned");

                node.current_leader_id = Some(node.id);
                node.current_term = term;
                node.status = NodeStatus::Leader;
            },
            LeaderElectionEvent::SetNodeToFollower(vr) => {
                let mut node = mutex_node.lock().expect("lock is poisoned");

                node.current_term = vr.term;
                node.status = NodeStatus::Follower;
            },
        }
    }
}

fn vote_request_processor(event_tx : Sender<LeaderElectionEvent>,
                          mutex_node: Arc<Mutex<Node>>,
                communicator : InProcNodeCommunicator,
                event_rx : Receiver<VoteRequest>) {

    loop {
        let request_result = event_rx.recv();
        let request = request_result.unwrap(); //TODO
        let node = mutex_node.lock().expect("lock is poisoned");

        println!("{:?}", request);
        if node.current_term < request.term { //TODO check logic
            communicator.send_vote_response(request.candidate_id, VoteResponse{voted_for_candidate_id : node.id, peer_id : node.id,  term : request.term})
        }
    }
}

fn notify_peers(term : u64,
                event_tx : Sender<LeaderElectionEvent>,
                node_id : u64,
                communicator : InProcNodeCommunicator,
                event_rx : Receiver<VoteResponse>,
                peers : Vec<u64>,
                quorum_size: u32) {
    let vote_request = VoteRequest { candidate_id: node_id, term };
    for peer_id in peers {
        communicator.send_vote_request(peer_id, vote_request);
    }

    let timeout = crossbeam_channel::after(random_awaiting_leader_duration_ms());

    let mut votes = 1;
    loop {
        select!(
            recv(timeout) -> _ => {
                println!("Leader election failed.");
                event_tx.send(LeaderElectionEvent::SetNodeToFollower(ElectionNotice{candidate_id : node_id, term}));
                return;
            },
            recv(event_rx) -> response => {
                let resp = response.unwrap(); //TODO

                println!("{:?}", resp);
                if (resp.term == term) && (resp.voted_for_candidate_id == node_id) {
                    votes += 1;

                    if votes >= quorum_size {
                        break;
                    }
                }
            }
        );
    }

    let event_promote_to_leader = LeaderElectionEvent::PromoteNodeToLeader(term);

    event_tx.send(event_promote_to_leader).expect("cannot promote to leader");
}


pub fn start(config : NodeConfiguration) {
    let (tx, rx): (Sender<LeaderElectionEvent>, Receiver<LeaderElectionEvent>) = crossbeam_channel::unbounded();
    let node = Node{id : config.node_id, current_term: 0, status : NodeStatus::Follower, current_leader_id: None};
    let mutex_node = Arc::new(Mutex::new(node));

    let run_thread_node_mutex = mutex_node.clone();
    let run_thread_event_sender = tx.clone();
    let run_thread_communicator = config.communicator.clone();
    let run_thread_response_rx_channel = config.response_rx_channel.clone();
    let quorum_size = config.quorum_size;
    let run_thread_peer_id_list = config.peers_id_list.clone();
    let run_thread = thread::spawn(move|| run_leader_election_process(run_thread_node_mutex,
                                                                      run_thread_event_sender,
                                                                      rx,
                                                                      run_thread_response_rx_channel,
                                                                      run_thread_communicator,
                                                                      run_thread_peer_id_list,
                                                                      quorum_size));

    let check_leader_thread_node_mutex = mutex_node.clone();
    let check_leader_thread_event_sender = tx.clone();
    let check_leader_thread = thread::spawn(move|| watch_leader_status(check_leader_thread_node_mutex,
                                                                       check_leader_thread_event_sender));


    let request_processor_thread_node_mutex = mutex_node.clone();
    let request_processor_thread_event_sender = tx.clone();
    let request_processor_thread_communicator = config.communicator.clone();
    let request_processor_thread_rx_channel = config.request_rx_channel.clone();
    let request_processor_thread = thread::spawn(move|| vote_request_processor(request_processor_thread_event_sender,
                                                                               request_processor_thread_node_mutex,
                                                                               request_processor_thread_communicator,
                                                                               request_processor_thread_rx_channel));


    let check_debug_node_thread = thread::spawn(move|| debug_node_status( mutex_node.clone()));

    let _ = check_debug_node_thread.join();
    let _ = request_processor_thread.join();
    let _ = check_leader_thread.join();
    let _ = run_thread.join();

}
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
    ResetNodeToFollower(ElectionNotice),
}

enum LeaderElectedEvent {
    ResetWatchdogCounter
}

struct ElectionNotice {
    pub term : u64,
    pub candidate_id : u64
}

#[derive(Debug)]
struct Node {
    id : u64,
    current_term: u64,
    current_leader_id: Option<u64>, //TODO delete?
    voted_for_id: Option<u64>,
    status : NodeStatus
}

fn debug_node_status(mutex_node: Arc<Mutex<Node>>) {
    loop {
        let node = mutex_node.lock().expect("lock is poisoned");
        println!("{:?}", node);

        sleep(Duration::from_millis(1000));
    }
}

fn watch_leader_status(mutex_node: Arc<Mutex<Node>>,
                       leadership_event_tx : Sender<LeaderElectionEvent>,
                       watchdog_event_rx : Receiver<LeaderElectedEvent>) {
    loop {
        let timeout = crossbeam_channel::after(random_awaiting_leader_duration_ms());
        select!(
            recv(timeout) -> msg => {},
            recv(watchdog_event_rx) -> msg => {
                let node = mutex_node.lock().expect("lock is poisoned");
                println!("Received reset watchdog for node_id {:?}", node.id);
                continue
            },
            //TODO : notify leadership
        );

        let node = mutex_node.lock().expect("lock is poisoned");

        if let NodeStatus::Follower = node.status {
            println!("Leader awaiting time elapsed. Starting new election.");

            let current_leader_id = node.current_leader_id.clone();

            if current_leader_id.is_none() || current_leader_id.unwrap() != node.id {
                let next_term = node.current_term + 1;
                let candidate_promotion = LeaderElectionEvent::PromoteNodeToCandidate(ElectionNotice { term: next_term, candidate_id: node.id });
                leadership_event_tx.send(candidate_promotion).expect("cannot promote to candidate");
            }
        }
    }
}

fn notify_peers_timeout_duration_ms() -> Duration{
    Duration::from_millis(1000)
}

fn random_awaiting_leader_duration_ms() -> Duration{
    let range_start = 1000;
    let range_stop = 4000;
    let mut rng = rand::thread_rng();

    Duration::from_millis(rng.gen_range(range_start, range_stop))
}


fn run_leader_election_process(mutex_node: Arc<Mutex<Node>>,
                               leader_election_event_tx : Sender<LeaderElectionEvent>,
                               leader_election_event_rx : Receiver<LeaderElectionEvent>,
                               response_event_rx : Receiver<VoteResponse>,
                               watchdog_event_tx : Sender<LeaderElectedEvent>,
                               communicator : InProcNodeCommunicator,
                               peers : Vec<u64>,
                               quorum_size : u32
                                ) {

    {
        let node = mutex_node.lock().expect("lock is poisoned");

        println!("Node {:?} started", node.id );
    }// mutex lock release

    loop {
        let event_result = leader_election_event_rx.recv();

        let event = event_result.expect("receiving from closed channel");

        match event {
            LeaderElectionEvent::PromoteNodeToCandidate(vr) => {
                let mut node = mutex_node.lock().expect("lock is poisoned");

                node.status = NodeStatus::Candidate;
                node.current_leader_id = None;
                let event_sender = leader_election_event_tx.clone();

                let node_id = node.id;
                node.voted_for_id = Some(node_id);
                let peers_copy = peers.clone();
                let communicator_copy = communicator.clone();
                let response_rx_copy = response_event_rx.clone();

  //TODO              optional abort channel for notifier

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
        //        node.voted_for_id = None;
                node.current_term = term;
                node.status = NodeStatus::Leader;
                watchdog_event_tx.send(LeaderElectedEvent::ResetWatchdogCounter); //TODO check send result
            },
            LeaderElectionEvent::ResetNodeToFollower(vr) => {
                let mut node = mutex_node.lock().expect("lock is poisoned");

                node.current_term = vr.term;
                node.current_leader_id = Some(vr.candidate_id);
                node.status = NodeStatus::Follower;
                watchdog_event_tx.send(LeaderElectedEvent::ResetWatchdogCounter); //TODO check send result
            },
        }
    }
}

fn vote_request_processor(  leader_election_event_tx : Sender<LeaderElectionEvent>,
                            mutex_node: Arc<Mutex<Node>>,
                            communicator : InProcNodeCommunicator,
                            request_event_rx : Receiver<VoteRequest>,
) {

    loop {
        let request_result = request_event_rx.recv();
        let request = request_result.unwrap(); //TODO
        let mut node = mutex_node.lock().expect("lock is poisoned");

        println!("Received request {:?}", request);
        let mut vote_granted = false;

        if node.current_term < request.term {
      //      if let None = node.voted_for_id {
                vote_granted = true;
                node.voted_for_id = Some(request.candidate_id);
                let follower_event = ElectionNotice { term: request.term, candidate_id: request.candidate_id };
                leader_election_event_tx.send(LeaderElectionEvent::ResetNodeToFollower(follower_event)); //TODO check send
            }
      //  }
        communicator.send_vote_response(request.candidate_id, VoteResponse { vote_granted, peer_id: node.id, term: request.term })
    }
}

fn notify_peers(term : u64,
                election_event_tx : Sender<LeaderElectionEvent>,
                node_id : u64,
                communicator : InProcNodeCommunicator,
                vote_response_event_rx : Receiver<VoteResponse>,
           //     abort_election_event_rx : Receiver<LeaderElectedEvent>,
                peers : Vec<u64>,
                quorum_size: u32) {
    let vote_request = VoteRequest { candidate_id: node_id, term };
    for peer_id in peers {
        communicator.send_vote_request(peer_id, vote_request);
    }

    let timeout = crossbeam_channel::after(notify_peers_timeout_duration_ms());

    let mut votes = 1;
    loop {
        select!(
            recv(timeout) -> _ => {
                println!("Leader election failed for {:?} ", node_id);
                election_event_tx.send(LeaderElectionEvent::ResetNodeToFollower(ElectionNotice{candidate_id : node_id, term})); //TODO send
                return;
            },
//            recv(abort_election_event_rx) -> _ => {
//                println!("Leader election received for {:?} ", node_id);
//                return;
//            },
            recv(vote_response_event_rx) -> response => {
                let resp = response.unwrap(); //TODO

                println!("receiving response {:?}", resp);
                if (resp.term == term) && (resp.vote_granted) {
                    votes += 1;

                    if votes >= quorum_size {
                        break;
                    }
                }
            }
        );
    }

    let event_promote_to_leader = LeaderElectionEvent::PromoteNodeToLeader(term);

    election_event_tx.send(event_promote_to_leader).expect("cannot promote to leader");
}


pub fn start(config : NodeConfiguration) {
    let (tx, rx): (Sender<LeaderElectionEvent>, Receiver<LeaderElectionEvent>) = crossbeam_channel::unbounded();
    let (reset_leadership_watchdog_tx, reset_leadership_watchdog_rx) : (Sender<LeaderElectedEvent>, Receiver<LeaderElectedEvent>) = crossbeam_channel::unbounded();

    let node = Node{id : config.node_id, current_term: 0, status : NodeStatus::Follower, current_leader_id: None, voted_for_id : None};
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
                                                                      reset_leadership_watchdog_tx,
                                                                      run_thread_communicator,
                                                                      run_thread_peer_id_list,
                                                                      quorum_size));


    let check_leader_thread_node_mutex = mutex_node.clone();
    let check_leader_thread_event_sender = tx.clone();
    let check_leader_thread = thread::spawn(move||
        watch_leader_status(check_leader_thread_node_mutex,check_leader_thread_event_sender, reset_leadership_watchdog_rx
    ));


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
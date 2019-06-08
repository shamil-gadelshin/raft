use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread::sleep;
use std::thread;
use crossbeam_channel::{Sender, Receiver};

pub mod communication;
mod core;
mod leader_watcher;
mod peer_notifier;
mod vote_request_processor;
mod election;

use self::core::*;
use self::communication::{VoteResponse, VoteRequest};
use self::communication::{InProcNodeCommunicator};


pub struct NodeConfiguration {
    pub node_id: u64,
    pub peers_id_list : Vec<u64>,
    pub quorum_size: u32,
    pub request_rx_channel : Receiver<VoteRequest>,
    pub response_rx_channel : Receiver<VoteResponse>,
    pub communicator : InProcNodeCommunicator
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

    let run_thread = thread::spawn(move|| election::run_leader_election_process(run_thread_node_mutex,
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
        leader_watcher::watch_leader_status(check_leader_thread_node_mutex,check_leader_thread_event_sender, reset_leadership_watchdog_rx
    ));


    let request_processor_thread_node_mutex = mutex_node.clone();
    let request_processor_thread_event_sender = tx.clone();
    let request_processor_thread_communicator = config.communicator.clone();
    let request_processor_thread_rx_channel = config.request_rx_channel.clone();
    let request_processor_thread = thread::spawn(move|| vote_request_processor::vote_request_processor(request_processor_thread_event_sender,
                                                                                                       request_processor_thread_node_mutex,
                                                                                                       request_processor_thread_communicator,
                                                                                                       request_processor_thread_rx_channel));


    let check_debug_node_thread = thread::spawn(move|| debug_node_status( mutex_node.clone()));

    let _ = check_debug_node_thread.join();
    let _ = request_processor_thread.join();
    let _ = check_leader_thread.join();
    let _ = run_thread.join();

}


//
//pub trait Election {
//    fn request_vote(&self, request : communication::VoteRequest);
//    fn poll_for_vote_response(&self) -> VoteResponse;
//
//    fn send_vote(&self, response : VoteResponse);
//    fn poll_for_vote_requests(&self) -> VoteRequest;
//
////   fn send_leader_heartbeat();
//}





fn debug_node_status(mutex_node: Arc<Mutex<Node>>) {
    loop {
//        let node_copy;
//        {
//            let node = mutex_node.lock().expect("lock is poisoned");
//            node_copy = node.clone();
//        }
//        print_event(format!("Node {:?}. {:?}", node_copy.id, node_copy));

        sleep(Duration::from_millis(1000));
    }
}





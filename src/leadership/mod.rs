use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread::sleep;
use std::thread;
use crossbeam_channel::{Sender, Receiver};

pub mod communication;
pub mod core; //TODO change project structure
mod leader_watcher;
mod peer_notifier;
mod vote_request_processor;
mod election;

use self::core::*;
use self::communication::{VoteResponse, VoteRequest, AppendEntriesRequest};
use self::communication::{InProcNodeCommunicator};
use crate::log_replication::*; //TODO change project structure
use crate::log_replication::append_entries_processor::*;

pub struct NodeConfiguration {
    pub node_id: u64,
    pub peers_id_list : Vec<u64>,
    pub quorum_size: u32,
    pub vote_request_rx_channel: Receiver<VoteRequest>,
    pub vote_response_rx_channel: Receiver<VoteResponse>,
    pub append_entries_rx_channel: Receiver<AppendEntriesRequest>,
    pub communicator : InProcNodeCommunicator
}

pub fn start(config : NodeConfiguration) {
    let (tx, rx): (Sender<LeaderElectionEvent>, Receiver<LeaderElectionEvent>) = crossbeam_channel::unbounded();
    let (reset_leadership_watchdog_tx, reset_leadership_watchdog_rx) : (Sender<LeaderConfirmationEvent>, Receiver<LeaderConfirmationEvent>) = crossbeam_channel::unbounded();

    let node = Node{id : config.node_id, current_term: 0, status : NodeStatus::Follower, current_leader_id: None, voted_for_id : None};
    let mutex_node = Arc::new(Mutex::new(node));

    let run_thread_node_mutex = mutex_node.clone();
    let run_thread_reset_leadership_watchdog_tx = reset_leadership_watchdog_tx.clone();
    let run_thread_event_sender = tx.clone();
    let run_thread_communicator = config.communicator.clone();
    let run_thread_response_rx_channel = config.vote_response_rx_channel.clone();
    let quorum_size = config.quorum_size;
    let run_thread_peer_id_list = config.peers_id_list.clone();

    let run_thread = thread::spawn(move|| election::run_leader_election_process(run_thread_node_mutex,
                                                                      run_thread_event_sender,
                                                                      rx,
                                                                      run_thread_response_rx_channel,
                                                                                run_thread_reset_leadership_watchdog_tx,
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
    let request_processor_thread_rx_channel = config.vote_request_rx_channel.clone();
    let request_processor_thread = thread::spawn(move|| vote_request_processor::vote_request_processor(request_processor_thread_event_sender,
                                                                                                       request_processor_thread_node_mutex,
                                                                                                       request_processor_thread_communicator,
                                                                                                       request_processor_thread_rx_channel));


    let debug_mutex_clone = mutex_node.clone();
    let check_debug_node_thread = thread::spawn(move|| debug_node_status( debug_mutex_clone));

    let append_entries_mutex_clone = mutex_node.clone();
    let append_entries_thread_peer_id_list = config.peers_id_list.clone();
    let append_entries_thread_communicator = config.communicator.clone();
    let append_entries_thread = thread::spawn(move|| append_entries( append_entries_mutex_clone, append_entries_thread_peer_id_list,append_entries_thread_communicator));


    let append_entries_procesor_mutex_clone = mutex_node.clone();
    let append_entries_procesor_thread_reset_leadership_watchdog_tx = reset_leadership_watchdog_tx.clone();

    let append_entries_processor_thread = thread::spawn(move|| append_entries_processor(
        append_entries_procesor_mutex_clone,
        config.append_entries_rx_channel,
        append_entries_procesor_thread_reset_leadership_watchdog_tx));

    let _ = append_entries_thread.join();
    let _ = append_entries_processor_thread.join();
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





fn debug_node_status(_: Arc<Mutex<Node>>) {
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





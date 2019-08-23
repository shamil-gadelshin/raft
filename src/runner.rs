use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread::sleep;
use std::thread;
use crossbeam_channel::{Sender, Receiver};

use crate::core::*;

use crate::communication::{InProcNodeCommunicator};

use crate::leadership::election::*;
use crate::leadership::leader_watcher ::*;
use crate::leadership::vote_request_processor ::*;

use crate::log_replication::append_entries_processor::*; //TODO change project structure
use crate::log_replication::append_entries_sender::*;

use crate::client_requests::*;
use crate::membership::*;

pub struct NodeConfiguration {
    pub node_id: u64,
    pub cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
    pub communicator : InProcNodeCommunicator,
    pub client_request_handler : ClientRequestHandler
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
    let run_thread_response_rx_channel = config.communicator.get_vote_response_channel_rx(config.node_id);
    let run_thread_cluster_configuration = config.cluster_configuration.clone();

    let run_thread = thread::spawn(move|| run_leader_election_process(run_thread_node_mutex,
                                                                                run_thread_event_sender,
                                                                                rx,
                                                                                run_thread_response_rx_channel,
                                                                                run_thread_reset_leadership_watchdog_tx,
                                                                                run_thread_communicator,
                                                                                run_thread_cluster_configuration));


    let check_leader_thread_node_mutex = mutex_node.clone();
    let check_leader_thread_event_sender = tx.clone();
    let check_leader_thread = thread::spawn(move||
        watch_leader_status(check_leader_thread_node_mutex,check_leader_thread_event_sender, reset_leadership_watchdog_rx
        ));


    let request_processor_thread_node_mutex = mutex_node.clone();
    let request_processor_thread_event_sender = tx.clone();
    let request_processor_thread_communicator = config.communicator.clone();
    let request_processor_thread_rx_channel = config.communicator.get_vote_request_channel_rx(config.node_id);
    let request_processor_thread = thread::spawn(move|| vote_request_processor(request_processor_thread_event_sender,
                                                                                                       request_processor_thread_node_mutex,
                                                                                                       request_processor_thread_communicator,
                                                                                                       request_processor_thread_rx_channel));


    let debug_mutex_clone = mutex_node.clone();
    let check_debug_node_thread = thread::spawn(move|| debug_node_status( debug_mutex_clone));

    let append_entries_mutex_clone = mutex_node.clone();
    let append_entries_thread_cluster_configuration = config.cluster_configuration.clone();
    let append_entries_thread_communicator = config.communicator.clone();
    let append_entries_thread = thread::spawn(move|| send_append_entries(append_entries_mutex_clone, append_entries_thread_cluster_configuration, append_entries_thread_communicator));


    let append_entries_processor_mutex_clone = mutex_node.clone();
    let append_entries_processor_thread_reset_leadership_watchdog_tx = reset_leadership_watchdog_tx.clone();
    let append_entries_request_rx = config.communicator.get_append_entries_request_rx(config.node_id);
    let append_entries_processor_thread = thread::spawn(move|| append_entries_processor(
        append_entries_processor_mutex_clone,
        append_entries_request_rx,
        append_entries_processor_thread_reset_leadership_watchdog_tx));

    let change_membership_mutex_clone = mutex_node.clone();
    let change_membership_cluster_config_clone = config.cluster_configuration.clone();
    let add_server_rx = config.client_request_handler.get_add_server_channel_rx();
    let change_membership_thread = thread::spawn(move|| change_membership(
        change_membership_mutex_clone,
        change_membership_cluster_config_clone,
        add_server_rx));

    let _ = append_entries_thread.join();
    let _ = append_entries_processor_thread.join();
    let _ = check_debug_node_thread.join();
    let _ = request_processor_thread.join();
    let _ = check_leader_thread.join();
    let _ = run_thread.join();

}

//TODO remove debug
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





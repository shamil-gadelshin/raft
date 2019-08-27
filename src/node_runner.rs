use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread::{sleep, JoinHandle};
use std::thread;
use crossbeam_channel::{Sender, Receiver};

use crate::common::{LeaderConfirmationEvent};
use crate::state::{Node, NodeStatus};
use crate::communication::client::{AddServerRequest};
use crate::configuration::node::{NodeConfiguration};


use crate::leadership::election::{LeaderElectionEvent, run_leader_election_process};
use crate::leadership::leader_watcher::{watch_leader_status};
use crate::leadership::vote_request_processor::{vote_request_processor};
use crate::operation_log::replication::append_entries_processor::{append_entries_processor};
use crate::operation_log::replication::append_entries_sender::{send_append_entries};
use crate::operation_log::storage::LogStorage;
use crate::membership::{change_membership};
use std::collections::HashMap;


//TODO check clones number - consider borrowing &
pub fn start_node<Log: Sync + Send + LogStorage + 'static>(node_config : NodeConfiguration, log_storage : Log) {
    let node = Node{id : node_config.node_id,
        current_term: 0,
        status : NodeStatus::Follower,
        current_leader_id: None,
        voted_for_id : None,
        log : log_storage,
        next_index : HashMap::new(),
        match_index : HashMap::new(),
    };

    let protected_node = Arc::new(Mutex::new(node));

    let (leader_election_tx, leader_election_rx): (Sender<LeaderElectionEvent>, Receiver<LeaderElectionEvent>) = crossbeam_channel::unbounded();
    let (reset_leadership_watchdog_tx, reset_leadership_watchdog_rx) : (Sender<LeaderConfirmationEvent>, Receiver<LeaderConfirmationEvent>) = crossbeam_channel::unbounded();
    let (append_entries_add_server_tx, append_entries_add_server_rx) : (Sender<AddServerRequest>, Receiver<AddServerRequest>) = crossbeam_channel::unbounded();
    let (leader_initial_heartbeat_tx, leader_initial_heartbeat_rx) : (Sender<bool>, Receiver<bool>) = crossbeam_channel::unbounded();

    let election_thread = create_election_thread(protected_node.clone(),
                                            &node_config,
                                            leader_election_rx.clone(),
                                            leader_election_tx.clone(),
                                                 leader_initial_heartbeat_tx,
                                            reset_leadership_watchdog_tx.clone(),
    );

    let check_leader_thread = create_check_leader_thread(protected_node.clone(),
                                                         leader_election_tx.clone(),
                                                         reset_leadership_watchdog_rx);

    let vote_request_processor_thread = create_vote_request_processor_thread(protected_node.clone(),
                                                                        leader_election_tx.clone(),
                                                                        &node_config
    );

    /* debug  */
    let debug_mutex_clone = protected_node.clone();
    let _ = thread::spawn(move|| debug_node_status( debug_mutex_clone));


    let append_entries_thread = create_append_entries_thread(protected_node.clone(),
                                                             append_entries_add_server_rx,
                                                             leader_initial_heartbeat_rx,
                                                             &node_config);

    let append_entries_processor_thread = create_append_entries_processor_thread(protected_node.clone(),
                                                                                 reset_leadership_watchdog_tx.clone(),
                                                                                 leader_election_tx.clone(),
                                                                                 &node_config);

    let change_membership_thread = create_change_membership_thread(protected_node.clone(),
                                                                   append_entries_add_server_tx,
                                                                   &node_config);


    info!("Node {:?} started", node_config.node_id);

    let _ = change_membership_thread.join();
    let _ = append_entries_thread.join();
    let _ = append_entries_processor_thread.join();
    let _ = vote_request_processor_thread.join();
    let _ = check_leader_thread.join();
    let _ = election_thread.join();
}

fn create_change_membership_thread<Log : LogStorage + Sync + Send+ 'static>(protected_node : Arc<Mutex<Node<Log>>>,
                                                                            append_entries_add_server_tx : Sender<AddServerRequest>,
                                                                            node_config : &NodeConfiguration) -> JoinHandle<()> {
    let cluster_config = node_config.cluster_configuration.clone();
    let client_add_server_request_rx = node_config.client_request_handler.get_add_server_request_rx();
    let client_add_server_response_tx = node_config.client_request_handler.get_add_server_response_tx();
    let change_membership_thread = thread::spawn(move|| change_membership(
        protected_node,
        cluster_config,
        client_add_server_request_rx,
        client_add_server_response_tx,
        append_entries_add_server_tx));

    change_membership_thread
}

fn create_append_entries_processor_thread<Log: Sync + Send + LogStorage + 'static>(protected_node : Arc<Mutex<Node<Log>>>,
                                                                                   reset_leadership_watchdog_tx : Sender<LeaderConfirmationEvent>,
                                                                                   leader_election_tx : Sender<LeaderElectionEvent>,
                                                                                   node_config : &NodeConfiguration) -> JoinHandle<()> {
    let append_entries_request_rx = node_config.peer_communicator.get_append_entries_request_rx(node_config.node_id);
    let append_entries_response_tx = node_config.peer_communicator.get_append_entries_response_tx(node_config.node_id);
    let append_entries_processor_thread = thread::spawn(move|| append_entries_processor(
        protected_node,
        leader_election_tx,
        append_entries_request_rx,
        append_entries_response_tx,
        reset_leadership_watchdog_tx));

    append_entries_processor_thread
}

fn create_append_entries_thread<Log : Sync + Send + LogStorage + 'static>(protected_node : Arc<Mutex<Node<Log>>>,
                                                                          append_entries_add_server_rx : Receiver<AddServerRequest>,
                                                                          leader_initial_heartbeat_rx : Receiver<bool>,
                                                                          node_config : &NodeConfiguration) -> JoinHandle<()> {

    let cluster_configuration = node_config.cluster_configuration.clone();
    let communicator = node_config.peer_communicator.clone();
    let append_entries_thread = thread::spawn(move|| send_append_entries(protected_node,
                                                                         cluster_configuration,
                                                                         append_entries_add_server_rx,
                                                                         leader_initial_heartbeat_rx,
                                                                         communicator));

    append_entries_thread
}

fn create_vote_request_processor_thread<Log: Sync + Send  + LogStorage + 'static >(protected_node : Arc<Mutex<Node<Log>>>,
                                                                                   leader_election_tx : Sender<LeaderElectionEvent>,
                                                                                   node_config : &NodeConfiguration) -> JoinHandle<()> {
    let communicator = node_config.peer_communicator.clone();
    let vote_request_channel_rx = node_config.peer_communicator.get_vote_request_rx(node_config.node_id);
    let vote_request_processor_thread = thread::spawn(move || vote_request_processor(leader_election_tx,
                                                                                protected_node,
                                                                                communicator,
                                                                                vote_request_channel_rx));

    vote_request_processor_thread
}

fn create_check_leader_thread<Log: Sync + Send + LogStorage + 'static>(protected_node : Arc<Mutex<Node<Log>>>,
                                                                       leader_election_tx : Sender<LeaderElectionEvent>,
                                                                       reset_leadership_watchdog_rx : Receiver<LeaderConfirmationEvent>) -> JoinHandle<()> {
    let check_leader_thread = thread::spawn(move||
        watch_leader_status(protected_node,leader_election_tx, reset_leadership_watchdog_rx
        ));

    check_leader_thread
}

fn create_election_thread<Log: Sync + Send + LogStorage + 'static>(protected_node : Arc<Mutex<Node<Log>>>,
                                                                   node_config : &NodeConfiguration,
                                                                   leader_election_rx : Receiver<LeaderElectionEvent>,
                                                                   leader_election_tx : Sender<LeaderElectionEvent>,
                                                                   leader_initial_heartbeat_tx : Sender<bool>,
                                                                   reset_leadership_watchdog_tx : Sender<LeaderConfirmationEvent>

) -> JoinHandle<()> {
    let vote_response_rx_channel = node_config.peer_communicator.get_vote_response_rx(node_config.node_id);
    let cluster_config = node_config.cluster_configuration.clone();
    let communicator = node_config.peer_communicator.clone();

    let run_thread = thread::spawn(move|| run_leader_election_process(protected_node,
                                                                      leader_election_tx,
                                                                      leader_election_rx,
                                                                      vote_response_rx_channel,
                                                                      leader_initial_heartbeat_tx,
                                                                      reset_leadership_watchdog_tx,
                                                                      communicator,
                                                                      cluster_config));

    run_thread
}

fn debug_node_status<Log: Sync + LogStorage + 'static>(_: Arc<Mutex<Node<Log>>>) {
    sleep(Duration::from_secs(1000));
    loop {
//        let node_copy;
//        {
//            let node = mutex_node.lock().expect("node ock is not poisoned");
//            node_copy = node.clone();
//        }
//        print_event(format!("Node {:?}. {:?}", node_copy.id, node_copy));

        sleep(Duration::from_millis(1000));
    }
}





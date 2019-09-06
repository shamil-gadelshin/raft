use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle};
use std::thread;

use crossbeam_channel::{Sender, Receiver};

use crate::common::{LeaderConfirmationEvent};
use crate::state::{Node, NodeStatus};
use crate::configuration::node::{NodeConfiguration};
use crate::leadership::election::{LeaderElectionEvent};
use crate::operation_log::storage::LogStorage;
use crate::fsm::{Fsm};
use crate::workers;

pub fn run_thread<Log: Sync + Send + LogStorage + 'static>(node_config : NodeConfiguration, log_storage : Log) -> JoinHandle<()>{
    thread::spawn(move || workers::node_main_process::start_node(node_config, log_storage))
}

//TODO check clones number - consider borrowing &
fn start_node<Log: Sync + Send + LogStorage + 'static>(node_config : NodeConfiguration, log_storage : Log) {

    add_this_node_to_cluster(&node_config);

    let (replicate_log_to_peer_tx, replicate_log_to_peer_rx) : (Sender<u64>, Receiver<u64>) = crossbeam_channel::unbounded();
    let fsm = Fsm::new(node_config.cluster_configuration.clone());
    let node = Node::new(node_config.node_id,
    None,
    NodeStatus::Follower,
    log_storage,
    fsm.clone(),
    node_config.peer_communicator.clone(),
    node_config.cluster_configuration.clone(),
    replicate_log_to_peer_tx.clone()
    );

    let protected_node = Arc::new(Mutex::new(node));

    let (leader_election_tx, leader_election_rx): (Sender<LeaderElectionEvent>, Receiver<LeaderElectionEvent>) = crossbeam_channel::unbounded();
    let (reset_leadership_watchdog_tx, reset_leadership_watchdog_rx) : (Sender<LeaderConfirmationEvent>, Receiver<LeaderConfirmationEvent>) = crossbeam_channel::unbounded();
    let (leader_initial_heartbeat_tx, leader_initial_heartbeat_rx) : (Sender<bool>, Receiver<bool>) = crossbeam_channel::unbounded();
    let (replicate_log_to_peer_rx, replicate_log_to_peer_tx) : (Sender<u64>, Receiver<u64>) = crossbeam_channel::unbounded();

    let election_thread = workers::election_manager::run_thread(protected_node.clone(),
                                                                &node_config,
                                                                leader_election_rx.clone(),
                                                                leader_election_tx.clone(),
                                                                leader_initial_heartbeat_tx,
                                                                reset_leadership_watchdog_tx.clone(),
    );



    let check_leader_thread = workers::leader_status_watcher::run_thread(protected_node.clone(),
                                                                         leader_election_tx.clone(),
                                                                         reset_leadership_watchdog_rx);

    let vote_request_processor_thread = workers::vote_request_processor::run_thread(protected_node.clone(),
                                                                                    leader_election_tx.clone(),
                                                                                    &node_config
    );

    let send_heartbeat_append_entries_thread = workers::leader_heartbeat_sender::run_thread(protected_node.clone(),
                                                                                            leader_initial_heartbeat_rx,
                                                                                            &node_config);

    let append_entries_processor_thread = workers::append_entries_processor::run_thread(protected_node.clone(),
                                                                                        reset_leadership_watchdog_tx.clone(),
                                                                                        leader_election_tx.clone(),
                                                                                        &node_config);

    let client_request_handler_thread = workers::client_request_handler::run_thread(protected_node.clone(),
                                                                                    &node_config);
    let peer_log_replicator_thread = workers::peer_log_replicator::run_thread(protected_node.clone(),
                                                                              replicate_log_to_peer_tx,
                                                                              replicate_log_to_peer_rx,
                                                                                    &node_config);

    info!("Node {:?} started", node_config.node_id);

    let _ = client_request_handler_thread.join();
    let _ = send_heartbeat_append_entries_thread.join();
    let _ = append_entries_processor_thread.join();
    let _ = vote_request_processor_thread.join();
    let _ = check_leader_thread.join();
    let _ = election_thread.join();
}

fn add_this_node_to_cluster(node_config: &NodeConfiguration) {
    let cluster_configuration = node_config.cluster_configuration.clone();
    let mut cluster = cluster_configuration.lock().expect("cluster lock is not poisoned");

    cluster.add_peer(node_config.node_id);
}











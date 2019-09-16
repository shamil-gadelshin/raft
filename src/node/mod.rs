use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle};
use std::thread;

use crossbeam_channel::{Sender, Receiver};

use crate::common::{LeaderConfirmationEvent, WorkerPool};
use crate::state::{Node, NodeStatus, NodeStateSaver};
use crate::configuration::node::{NodeConfiguration, ElectionTimer};
use crate::leadership::node_leadership_status::{LeaderElectionEvent, ElectionManagerParams, run_node_status_watcher};
use crate::operation_log::OperationLog;
use crate::fsm::{FiniteStateMachine};
use crate::{node, common, PeerRequestChannels, PeerRequestHandler, Worker};
use crate::request_handler::client::{ClientRequestHandlerParams, process_client_requests};
use crate::operation_log::replication::heartbeat_sender::{SendHeartbeatAppendEntriesParams, send_heartbeat_append_entries};
use crate::leadership::leader_watcher::{WatchLeaderStatusParams, watch_leader_status};
use crate::operation_log::replication::peer_log_replicator::{LogReplicatorParams, replicate_log_to_peer};
use crate::fsm::updater::{update_fsm, FsmUpdaterParams};
use crate::request_handler::peer::{PeerRequestHandlerParams, process_peer_request};
use crate::communication::client::ClientRequestChannels;


//TODO refactor to generic worker
pub fn start<Log, Fsm, Cc, Pc, Et, Ns>(node_config : NodeConfiguration<Cc, Pc, Et>,
                                       log_storage : Log,
                                       fsm : Fsm,
                                       state_saver : Ns,
                                       terminate_worker_rx : Receiver<()>) -> JoinHandle<()>
    where Log: OperationLog ,
          Fsm: FiniteStateMachine,
          Cc : ClientRequestChannels,
          Pc : PeerRequestHandler + PeerRequestChannels,
          Et : ElectionTimer,
          Ns : NodeStateSaver {
    thread::spawn(move || node::start_node(node_config, log_storage, fsm, state_saver, terminate_worker_rx))
}

//TODO check clones number - consider borrowing &
fn start_node<Log, Fsm, Cc, Pc, Et, Ns>(node_config : NodeConfiguration<Cc, Pc, Et>,
                                        log_storage : Log,
                                        fsm : Fsm,
                                        state_saver : Ns,
                                        terminate_worker_rx : Receiver<()>)
    where Log: OperationLog ,
          Fsm: FiniteStateMachine,
          Cc : ClientRequestChannels,
          Pc : PeerRequestHandler + PeerRequestChannels,
          Et : ElectionTimer,
          Ns : NodeStateSaver {
    add_this_node_to_cluster(&node_config);

    let (replicate_log_to_peer_tx, replicate_log_to_peer_rx): (Sender<u64>, Receiver<u64>) = crossbeam_channel::unbounded();
    let (commit_index_updated_tx, commit_index_updated_rx): (Sender<u64>, Receiver<u64>) = crossbeam_channel::unbounded();
    let node = Node::new(node_config.node_state.node_id,
                         node_config.node_state.current_term,
                         node_config.node_state.vote_for_id,
                         NodeStatus::Follower,
                         log_storage,
                         fsm,
                         node_config.peer_communicator.clone(),
                         node_config.cluster_configuration.clone(),
                         replicate_log_to_peer_tx.clone(),
                         commit_index_updated_tx,
                         state_saver
    );

    let protected_node = Arc::new(Mutex::new(node));

    let (leader_election_tx, leader_election_rx): (Sender<LeaderElectionEvent>, Receiver<LeaderElectionEvent>) = crossbeam_channel::unbounded();
    let (reset_leadership_watchdog_tx, reset_leadership_watchdog_rx): (Sender<LeaderConfirmationEvent>, Receiver<LeaderConfirmationEvent>) = crossbeam_channel::unbounded();
    let (leader_initial_heartbeat_tx, leader_initial_heartbeat_rx): (Sender<bool>, Receiver<bool>) = crossbeam_channel::unbounded();

    let election_worker = common::run_worker(
        run_node_status_watcher,
        ElectionManagerParams {
            protected_node: protected_node.clone(),
            leader_election_event_tx: leader_election_tx.clone(),
            leader_election_event_rx: leader_election_rx.clone(),
            leader_initial_heartbeat_tx,
            watchdog_event_tx: reset_leadership_watchdog_tx.clone(),
            peer_communicator: node_config.peer_communicator.clone(),
            cluster_configuration: node_config.cluster_configuration.clone(),
        });

    let check_leader_worker = common::run_worker(
        watch_leader_status,
        WatchLeaderStatusParams {
            protected_node: protected_node.clone(),
            leader_election_event_tx: leader_election_tx.clone(),
            watchdog_event_rx: reset_leadership_watchdog_rx,
            election_timer: node_config.election_timer
        });

    let peer_request_processor_worker = common::run_worker(
        process_peer_request,
        PeerRequestHandlerParams {
            protected_node: protected_node.clone(),
            leader_election_event_tx: leader_election_tx.clone(),
            reset_leadership_watchdog_tx: reset_leadership_watchdog_tx.clone(),
            peer_communicator: node_config.peer_communicator.clone()
        });

    let send_heartbeat_append_entries_worker = common::run_worker(
        send_heartbeat_append_entries,
        SendHeartbeatAppendEntriesParams {
            protected_node: protected_node.clone(),
            cluster_configuration: node_config.cluster_configuration.clone(),
            communicator: node_config.peer_communicator.clone(),
            leader_initial_heartbeat_rx
        });

    let client_request_handler_worker = common::run_worker(
        process_client_requests,
        ClientRequestHandlerParams {
            protected_node: protected_node.clone(),
            client_communicator: node_config.client_communicator.clone()
        });

    let peer_log_replicator_worker = common::run_worker(
        replicate_log_to_peer,
        LogReplicatorParams {
            protected_node: protected_node.clone(),
            replicate_log_to_peer_rx,
            replicate_log_to_peer_tx,
            communicator: node_config.peer_communicator.clone()
        });

    let fsm_updater_worker = common::run_worker(
        update_fsm,
        FsmUpdaterParams {
            protected_node: protected_node.clone(),
            commit_index_updated_rx
        });

    let workers = vec![client_request_handler_worker,
                       send_heartbeat_append_entries_worker,
                       peer_request_processor_worker,
                       check_leader_worker,
                       election_worker,
                       peer_log_replicator_worker,
                       fsm_updater_worker];

    let worker_pool = WorkerPool::new(workers);

    info!("Node {:?} started", node_config.node_state.node_id);

    let terminate_result = terminate_worker_rx.recv(); //TODO Check result

    info!("Node {:?} termination requested", node_config.node_state.node_id);

    worker_pool.terminate();
    worker_pool.join();

    info!("Node {:?} shutting down", node_config.node_state.node_id);
}


//TODO check cluster configuration functionality
fn add_this_node_to_cluster<Cc, Pc, Et>(node_config: &NodeConfiguration<Cc, Pc, Et>)
where Cc : ClientRequestChannels,
      Pc : Clone + PeerRequestHandler + PeerRequestChannels,
      Et : ElectionTimer{
    let cluster_configuration = node_config.cluster_configuration.clone();
    let mut cluster = cluster_configuration.lock().expect("cluster lock is not poisoned");

    cluster.add_peer(node_config.node_state.node_id);
}











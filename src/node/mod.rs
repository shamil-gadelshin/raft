pub mod configuration;
pub mod state;

use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, Sender};

use crate::common::RaftWorkerPool;
use crate::communication::client::ClientRequestChannels;
use crate::leadership::node_leadership_fsm::run_node_status_watcher;
use crate::leadership::node_leadership_fsm::{ElectionManagerParams, LeaderElectionEvent};
use crate::node::state::{Node, NodeStateSaver};
use crate::operation_log::replication::heartbeat_sender::send_heartbeat_append_entries;
use crate::operation_log::replication::heartbeat_sender::SendHeartbeatAppendEntriesParams;
use crate::operation_log::replication::peer_log_replicator::replicate_log_to_peer;
use crate::operation_log::replication::peer_log_replicator::LogReplicatorParams;
use crate::operation_log::OperationLog;
use crate::request_handler::client::{process_client_requests, ClientRequestHandlerParams};
use crate::request_handler::peer::{process_peer_request, PeerRequestHandlerParams};
use crate::rsm::updater::{update_rsm, RsmUpdaterParams};
use crate::rsm::ReplicatedStateMachine;
use crate::{
    common, Cluster, ElectionTimer, NodeConfiguration, PeerRequestChannels, PeerRequestHandler,
};
use crate::leadership::watchdog::leader_status_watcher::{watch_leader_status};
use crate::leadership::watchdog::leader_status_watcher::{WatchLeaderStatusParams};
use crate::leadership::watchdog::watchdog_handler::LeadershipStatusWatchdogHandler;

pub fn start<Log, Rsm, Cc, Pc, Et, Ns, Cl>(
    node_config : NodeConfiguration<Log, Rsm, Cc, Pc, Et, Ns, Cl>,
    terminate_worker_rx : Receiver<()>)
    where Log: OperationLog ,
          Rsm: ReplicatedStateMachine,
          Cc : ClientRequestChannels,
          Pc : PeerRequestHandler + PeerRequestChannels,
          Et : ElectionTimer,
          Ns : NodeStateSaver,
          Cl : Cluster {

    let (replicate_log_to_peer_tx, replicate_log_to_peer_rx): (Sender<u64>, Receiver<u64>) =
        crossbeam_channel::unbounded();
    let (commit_index_updated_tx, commit_index_updated_rx): (Sender<u64>, Receiver<u64>) =
        crossbeam_channel::unbounded();

    let node = Node::new(
        node_config.node_state,
        node_config.operation_log,
        node_config.rsm,
        node_config.peer_communicator.clone(),
        node_config.cluster_configuration.clone(),
        node_config.state_saver,
        replicate_log_to_peer_tx.clone(),
        commit_index_updated_tx,
    );

    let protected_node = Arc::new(Mutex::new(node));

    let (leader_election_tx, leader_election_rx):
        (Sender<LeaderElectionEvent>, Receiver<LeaderElectionEvent>) =
        crossbeam_channel::unbounded();

    let leadership_watchdog_handler = LeadershipStatusWatchdogHandler::new();

    let (leader_initial_heartbeat_tx, leader_initial_heartbeat_rx)
        : (Sender<()>, Receiver<()>) = crossbeam_channel::unbounded();

    let election_worker = common::run_worker(
        run_node_status_watcher,
        ElectionManagerParams {
            protected_node: protected_node.clone(),
            leader_election_event_tx: leader_election_tx.clone(),
            leader_election_event_rx: leader_election_rx.clone(),
            leader_initial_heartbeat_tx,
            leadership_status_watchdog_handler: leadership_watchdog_handler.clone(),
            peer_communicator: node_config.peer_communicator.clone(),
            cluster_configuration: node_config.cluster_configuration.clone(),
        });

    let check_leader_worker = common::run_worker(
        watch_leader_status,
        WatchLeaderStatusParams {
            protected_node: protected_node.clone(),
            leader_election_event_tx: leader_election_tx.clone(),
            watchdog_event_rx: leadership_watchdog_handler.clone(),
            election_timer: node_config.election_timer
        });

    let peer_request_processor_worker = common::run_worker(
        process_peer_request,
        PeerRequestHandlerParams {
            protected_node: protected_node.clone(),
            leader_election_event_tx: leader_election_tx.clone(),
            leadership_status_watchdog_handler: leadership_watchdog_handler,
            peer_communicator: node_config.peer_communicator.clone(),
            communication_timeout: node_config.limits.communication_timeout
        });

    let send_heartbeat_append_entries_worker = common::run_worker(
        send_heartbeat_append_entries,
        SendHeartbeatAppendEntriesParams {
            protected_node: protected_node.clone(),
            cluster_configuration: node_config.cluster_configuration.clone(),
            communicator: node_config.peer_communicator.clone(),
            leader_initial_heartbeat_rx,
            heartbeat_timeout: node_config.limits.heartbeat_timeout
        });

    let client_request_handler_worker = common::run_worker(
        process_client_requests,
        ClientRequestHandlerParams {
            protected_node: protected_node.clone(),
            cluster_configuration: node_config.cluster_configuration.clone(),
            client_communicator: node_config.client_communicator.clone(),
            max_data_content_size: node_config.limits.max_data_content_size
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
        update_rsm,
        RsmUpdaterParams {
            protected_node: protected_node.clone(),
            commit_index_updated_rx
        });

    let workers = vec![
        client_request_handler_worker,
        send_heartbeat_append_entries_worker,
        peer_request_processor_worker,
        check_leader_worker,
        election_worker,
        peer_log_replicator_worker,
        fsm_updater_worker];

    let worker_pool = RaftWorkerPool::new(workers);

    info!("Node {} started", node_config.node_state.node_id);

    let terminate_result = terminate_worker_rx.recv();
    if let Err(e) = terminate_result {
        error!("Abnormal exit for node: {}", e);
    }

    info!("Node {} termination requested", node_config.node_state.node_id);

    worker_pool.terminate();
    worker_pool.join();

    info!("Node {} shutting down", node_config.node_state.node_id);
}










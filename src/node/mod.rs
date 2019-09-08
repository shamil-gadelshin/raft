use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle};
use std::thread;

use crossbeam_channel::{Sender, Receiver};

use crate::common::{LeaderConfirmationEvent};
use crate::state::{Node, NodeStatus};
use crate::configuration::node::{NodeConfiguration};
use crate::leadership::election::{LeaderElectionEvent, ElectionManagerParams, run_leader_election_process};
use crate::operation_log::LogStorage;
use crate::fsm::{Fsm};
use crate::{node, common};
use crate::request_handler::client::{ClientRequestHandlerParams, process_client_requests};
use crate::operation_log::replication::append_entries_processor::{append_entries_processor, AppendEntriesProcessorParams};
use crate::operation_log::replication::heartbeat_append_entries_sender::{SendHeartbeatAppendEntriesParams, send_heartbeat_append_entries};
use crate::leadership::leader_watcher::{WatchLeaderStatusParams, watch_leader_status};
use crate::operation_log::replication::peer_log_replicator::{LogReplicatorParams, replicate_log_to_peer};
use crate::leadership::vote_request_processor::{VoteRequestProcessorParams, vote_request_processor};
use crate::fsm::updater::{update_fsm, FsmUpdaterParams};


//TODO refactor to generic worker
pub fn start<Log: Sync + Send + LogStorage + 'static, FsmT:  Sync + Send + Fsm+ 'static>(node_config : NodeConfiguration, log_storage : Log, fsm : FsmT) -> JoinHandle<()>{
    thread::spawn(move || node::start_node(node_config, log_storage, fsm))
}

//TODO check clones number - consider borrowing &
fn start_node<Log: Sync + Send + LogStorage + 'static, FsmT:  Sync + Send +  Fsm+ 'static>(node_config : NodeConfiguration, log_storage : Log, fsm : FsmT) {
    add_this_node_to_cluster(&node_config);

    let (replicate_log_to_peer_tx, replicate_log_to_peer_rx): (Sender<u64>, Receiver<u64>) = crossbeam_channel::unbounded();
    let (commit_index_updated_tx, commit_index_updated_rx ): (Sender<u64>, Receiver<u64>) = crossbeam_channel::unbounded();
    let node = Node::new(node_config.node_id,
                         None,
                         NodeStatus::Follower,
                         log_storage,
                         fsm,
                         node_config.peer_communicator.clone(),
                         node_config.cluster_configuration.clone(),
                         replicate_log_to_peer_tx.clone(),
        commit_index_updated_tx
    );

    let protected_node = Arc::new(Mutex::new(node));

    let (leader_election_tx, leader_election_rx): (Sender<LeaderElectionEvent>, Receiver<LeaderElectionEvent>) = crossbeam_channel::unbounded();
    let (reset_leadership_watchdog_tx, reset_leadership_watchdog_rx): (Sender<LeaderConfirmationEvent>, Receiver<LeaderConfirmationEvent>) = crossbeam_channel::unbounded();
    let (leader_initial_heartbeat_tx, leader_initial_heartbeat_rx): (Sender<bool>, Receiver<bool>) = crossbeam_channel::unbounded();

    let election_thread = common::run_worker_thread(
        run_leader_election_process,
        ElectionManagerParams {
            protected_node: protected_node.clone(),
            leader_election_event_tx: leader_election_tx.clone(),
            leader_election_event_rx: leader_election_rx.clone(),
            leader_initial_heartbeat_tx,
            watchdog_event_tx: reset_leadership_watchdog_tx.clone(),
            communicator: node_config.peer_communicator.clone(),
            cluster_configuration: node_config.cluster_configuration.clone(),
        });

    let check_leader_thread = common::run_worker_thread(
        watch_leader_status,
        WatchLeaderStatusParams {
            protected_node: protected_node.clone(),
            leader_election_event_tx: leader_election_tx.clone(),
            watchdog_event_rx: reset_leadership_watchdog_rx
        });

    let vote_request_processor_thread = common::run_worker_thread(
        vote_request_processor,
        VoteRequestProcessorParams {
            protected_node: protected_node.clone(),
            leader_election_event_tx: leader_election_tx.clone(),
            request_event_rx: node_config.peer_communicator.get_vote_request_rx(node_config.node_id),
            communicator: node_config.peer_communicator.clone()
        });

    let send_heartbeat_append_entries_thread = common::run_worker_thread(
        send_heartbeat_append_entries,
        SendHeartbeatAppendEntriesParams {
            protected_node: protected_node.clone(),
            cluster_configuration: node_config.cluster_configuration.clone(),
            communicator: node_config.peer_communicator.clone(),
            leader_initial_heartbeat_rx
        });

    let append_entries_processor_thread = common::run_worker_thread(
        append_entries_processor,
        AppendEntriesProcessorParams {
            protected_node: protected_node.clone(),
            append_entries_request_rx: node_config.peer_communicator.get_append_entries_request_rx(node_config.node_id),
            append_entries_response_tx: node_config.peer_communicator.get_append_entries_response_tx(node_config.node_id),
            reset_leadership_watchdog_tx: reset_leadership_watchdog_tx.clone(),
            leader_election_event_tx: leader_election_tx.clone()
        }
    );

    let client_request_handler_thread = common::run_worker_thread(
        process_client_requests,
        ClientRequestHandlerParams {
            protected_node: protected_node.clone(),
            client_communicator: node_config.client_communicator.clone()
        });

    let peer_log_replicator_thread = common::run_worker_thread(
        replicate_log_to_peer,
        LogReplicatorParams {
            protected_node: protected_node.clone(),
            replicate_log_to_peer_rx,
            replicate_log_to_peer_tx,
            communicator: node_config.peer_communicator.clone()
        });

    let fsm_updater_thread = common::run_worker_thread(
        update_fsm,
        FsmUpdaterParams {
            protected_node: protected_node.clone(),
            commit_index_updated_rx
        });

    info!("Node {:?} started", node_config.node_id);

    let _ = client_request_handler_thread.join();
    let _ = send_heartbeat_append_entries_thread.join();
    let _ = append_entries_processor_thread.join();
    let _ = vote_request_processor_thread.join();
    let _ = check_leader_thread.join();
    let _ = election_thread.join();
    let _ = peer_log_replicator_thread.join();
    let _ = fsm_updater_thread.join();
}

fn add_this_node_to_cluster(node_config: &NodeConfiguration) {
    let cluster_configuration = node_config.cluster_configuration.clone();
    let mut cluster = cluster_configuration.lock().expect("cluster lock is not poisoned");

    cluster.add_peer(node_config.node_id);
}











use crossbeam_channel::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use crate::communication::peers::PeerRequestHandler;
use crate::leadership::election::{start_election, StartElectionParams};
use crate::leadership::LeaderConfirmationEvent;
use crate::node::state::{Node, NodeStateSaver, NodeStatus};
use crate::operation_log::OperationLog;
use crate::rsm::ReplicatedStateMachine;
use crate::{common, Cluster};

pub enum LeaderElectionEvent {
    PromoteNodeToCandidate(ElectionNotice),
    PromoteNodeToLeader(u64), //term
    ResetNodeToFollower(u64), //term
}

pub struct ElectionNotice {
    pub term: u64,
    pub candidate_id: u64,
}

pub struct ElectionManagerParams<Log, Rsm, Pc, Ns, Cl>
where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Pc: PeerRequestHandler,
    Ns: NodeStateSaver,
    Cl: Cluster,
{
    pub protected_node: Arc<Mutex<Node<Log, Rsm, Pc, Ns, Cl>>>,
    pub leader_election_event_tx: Sender<LeaderElectionEvent>,
    pub leader_election_event_rx: Receiver<LeaderElectionEvent>,
    pub leader_initial_heartbeat_tx: Sender<bool>,
    pub watchdog_event_tx: Sender<LeaderConfirmationEvent>,
    pub peer_communicator: Pc,
    pub cluster_configuration: Cl,
}

pub fn run_node_status_watcher<Log, Rsm, Pc, Ns, Cl>(
    params: ElectionManagerParams<Log, Rsm, Pc, Ns, Cl>,
    terminate_worker_rx: Receiver<()>,
) where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Pc: PeerRequestHandler,
    Ns: NodeStateSaver,
    Cl: Cluster,
{
    info!("Leader election status watcher worker started");
    loop {
        select!(
            recv(terminate_worker_rx) -> res  => {
                if res.is_err() {
                    error!("Abnormal exit for leader election status watcher worker");
                }
                break
            },
            recv(params.leader_election_event_rx) -> event_result => {
                let event = event_result.expect("can receive election event from channel");
                change_node_leadership_state(&params, event);
            }
        );
    }
    info!("Leader election status watcher worker stopped");
}

fn change_node_leadership_state<Log, Rsm, Pc, Ns, Cl>(
    params: &ElectionManagerParams<Log, Rsm, Pc, Ns, Cl>,
    event: LeaderElectionEvent,
) where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Pc: PeerRequestHandler,
    Ns: NodeStateSaver,
    Cl: Cluster,
{
    match event {
        LeaderElectionEvent::PromoteNodeToCandidate(vr) => {
            let mut node = params
                .protected_node
                .lock()
                .expect("node lock is not poisoned");

            let node_id = node.id;
            node.set_voted_for_id(Some(node_id));
            node.current_leader_id = None;
            node.status = NodeStatus::Candidate;

            info!(
                "Node {} Status changed to Candidate for term {}",
                node.id, vr.term
            );

            let peers_copy = params.cluster_configuration.get_peers(node_id);
            let quorum_size = params.cluster_configuration.get_quorum_size();

            let params = StartElectionParams {
                node_id,
                actual_current_term: node.get_current_term() - 1,
                next_term: vr.term,
                last_log_index: node.log.get_last_entry_index(),
                last_log_term: node.log.get_last_entry_term(),
                leader_election_event_tx: params.leader_election_event_tx.clone(),
                peers: peers_copy,
                quorum_size,
                peer_communicator: params.peer_communicator.clone(),
            };

            common::run_worker_thread(start_election, params);
        }
        LeaderElectionEvent::PromoteNodeToLeader(term) => {
            let mut node = params
                .protected_node
                .lock()
                .expect("node lock is not poisoned");

            node.current_leader_id = Some(node.id);
            node.set_current_term(term);
            node.status = NodeStatus::Leader;
            node.set_voted_for_id(None);

            info!(
                "Node {} Status changed to Leader for term {}",
                node.id, term
            );

            params
                .watchdog_event_tx
                .send(LeaderConfirmationEvent::ResetWatchdogCounter)
                .expect("can send LeaderElectedEvent");

            params
                .leader_initial_heartbeat_tx
                .send(true)
                .expect("can send leader initial heartbeat");
        }
        LeaderElectionEvent::ResetNodeToFollower(term) => {
            let mut node = params.protected_node.lock().expect("node lock is poisoned");

            node.set_current_term(term);
            node.status = NodeStatus::Follower;
            node.set_voted_for_id(None);
            info!(
                "Node {} Status changed to Follower for term {}",
                node.id, term
            );

            params
                .watchdog_event_tx
                .send(LeaderConfirmationEvent::ResetWatchdogCounter)
                .expect("can send LeaderConfirmationEvent");
        }
    }
}

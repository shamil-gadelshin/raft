use std::sync::{Arc, Mutex};
use crossbeam_channel::{Sender, Receiver};

use crate::common::{LeaderConfirmationEvent};
use crate::state::{Node, NodeStatus, NodeStateSaver};
use crate::communication::peers::{PeerRequestHandler};
use crate::configuration::cluster::{ClusterConfiguration};
use crate::common;
use crate::operation_log::OperationLog;
use crate::fsm::FiniteStateMachine;
use crate::leadership::election::{StartElectionParams, start_election};

pub enum LeaderElectionEvent {
    PromoteNodeToCandidate(ElectionNotice),
    PromoteNodeToLeader(u64),
    ResetNodeToFollower(ElectionNotice),
}

pub struct ElectionNotice {
    pub term: u64,
    pub candidate_id : u64 //TODO introduce options Candidate, Leader
}


pub struct ElectionManagerParams<Log, Fsm,Pc, Ns>
    where Log: OperationLog,
          Fsm: FiniteStateMachine,
          Pc : PeerRequestHandler,
          Ns : NodeStateSaver{
    pub protected_node: Arc<Mutex<Node<Log, Fsm, Pc, Ns>>>,
    pub leader_election_event_tx : Sender<LeaderElectionEvent>,
    pub leader_election_event_rx : Receiver<LeaderElectionEvent>,
    pub leader_initial_heartbeat_tx : Sender<bool>,
    pub watchdog_event_tx : Sender<LeaderConfirmationEvent>,
    pub peer_communicator: Pc,
    pub cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
}


pub fn run_node_status_watcher<Log, Fsm, Pc, Ns>(params : ElectionManagerParams<Log, Fsm, Pc, Ns>)
    where Log: OperationLog,
          Fsm: FiniteStateMachine,
          Pc : PeerRequestHandler,
          Ns : NodeStateSaver{
    loop {
        let event_result = params.leader_election_event_rx.recv();

        let event = event_result.expect("can receive election event from channel");

        match event {
            LeaderElectionEvent::PromoteNodeToCandidate(vr) => {
                let mut node = params.protected_node.lock().expect("node lock is not poisoned");

                let node_id = node.id;
                node.set_voted_for_id(Some(node_id));
                node.current_leader_id = None;
                node.status = NodeStatus::Candidate;
                info!("Node {:?} Status changed to Candidate", node.id);

                let cluster = params.cluster_configuration.lock().expect("node lock is not poisoned");

                let (peers_copy, quorum_size) =
                    (cluster.get_peers(node_id), cluster.get_quorum_size());

                let params = StartElectionParams{node_id,
                             actual_current_term: node.get_current_term() - 1,
                             next_term :vr.term,
                             last_log_index: node.log.get_last_entry_index(),
                             last_log_term: node.log.get_last_entry_term() ,
                             leader_election_event_tx: params.leader_election_event_tx.clone(),
                             peers: peers_copy,
                             quorum_size,
                             peer_communicator: params.peer_communicator.clone()};

                common::run_worker_thread(start_election, params);
            },
            LeaderElectionEvent::PromoteNodeToLeader(term) => {
                let mut node = params.protected_node.lock().expect("node lock is not poisoned");

                node.current_leader_id = Some(node.id);
                node.set_current_term(term);
                node.set_voted_for_id(None);
                node.status = NodeStatus::Leader;
                info!("Node {:?} Status changed to Leader", node.id);

                params.watchdog_event_tx.send(LeaderConfirmationEvent::ResetWatchdogCounter).expect("can send LeaderElectedEvent");
                params.leader_initial_heartbeat_tx.send(true).expect("can send leader initial heartbeat");
            },
            LeaderElectionEvent::ResetNodeToFollower(vr) => {
                let mut node = params.protected_node.lock().expect("node lock is poisoned");

                node.set_current_term(vr.term);
                node.status = NodeStatus::Follower;
                node.set_voted_for_id(None);
                info!("Node {:?} Status changed to Follower", node.id);

                params. watchdog_event_tx.send(LeaderConfirmationEvent::ResetWatchdogCounter).expect("can send LeaderConfirmationEvent");
            },
        }
    }
}
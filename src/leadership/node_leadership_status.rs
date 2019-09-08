use std::sync::{Arc, Mutex};
use crossbeam_channel::{Sender, Receiver};
use std::thread;

use crate::common::{LeaderConfirmationEvent};
use crate::state::{Node, NodeStatus};
use crate::communication::peers::{InProcPeerCommunicator, VoteRequest};
use crate::configuration::cluster::{ClusterConfiguration};
use super::election;
use crate::operation_log::LogStorage;
use crate::fsm::Fsm;

pub enum LeaderElectionEvent {
    PromoteNodeToCandidate(ElectionNotice),
    PromoteNodeToLeader(u64),
    ResetNodeToFollower(ElectionNotice),
}

pub struct ElectionNotice {
    pub term: u64,
    pub candidate_id : u64 //TODO introduce options Candidate, Leader
}


pub struct ElectionManagerParams<Log, FsmT>
    where Log: Sync + Send + LogStorage + 'static, FsmT: Sync + Send + Fsm + 'static {
    pub protected_node: Arc<Mutex<Node<Log, FsmT>>>,
    pub leader_election_event_tx : Sender<LeaderElectionEvent>,
    pub leader_election_event_rx : Receiver<LeaderElectionEvent>,
    pub leader_initial_heartbeat_tx : Sender<bool>,
    pub watchdog_event_tx : Sender<LeaderConfirmationEvent>,
    pub communicator : InProcPeerCommunicator,
    pub cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
}


pub fn run_node_status_watcher<Log: Sync + Send + LogStorage, FsmT:  Sync + Send + Fsm>(params : ElectionManagerParams<Log, FsmT>)
    where Log: Sync + Send + LogStorage + 'static, FsmT: Sync + Send + Fsm + 'static {
    loop {
        let event_result = params.leader_election_event_rx.recv();

        let event = event_result.expect("can receive election event from channel");

        match event {
            LeaderElectionEvent::PromoteNodeToCandidate(vr) => {
                let mut node = params.protected_node.lock().expect("node lock is not poisoned");

                let node_id = node.id;
                node.voted_for_id = Some(node_id);
                node.current_leader_id = None;
                node.status = NodeStatus::Candidate;
                info!("Node {:?} Status changed to Candidate", node.id);

                let cluster = params.cluster_configuration.lock().expect("node lock is not poisoned");

                let (peers_copy, quorum_size) =
                    (cluster.get_peers(node_id), cluster.get_quorum_size());


                let communicator_copy = params.communicator.clone();
                let election_event_tx_copy = params.leader_election_event_tx.clone();
                let actual_current_term = node.get_current_term() - 1; //TODO refactor

                let vote_request = VoteRequest {
                    candidate_id: node_id,
                    term:vr.term,
                    last_log_index:  node.log.get_last_entry_index() as u64,
                    last_log_term: node.log.get_last_entry_term() };

                //TODO spawn a worker
                thread::spawn(move || election::start_election(
                    actual_current_term,
                    election_event_tx_copy,
                    node_id,
                    communicator_copy,
                    peers_copy,
                    quorum_size,
                    vote_request));
            },
            LeaderElectionEvent::PromoteNodeToLeader(term) => {
                let mut node = params.protected_node.lock().expect("node lock is not poisoned");

                node.current_leader_id = Some(node.id);
                node.set_current_term(term);
                node.voted_for_id = None;
                node.status = NodeStatus::Leader;
                info!("Node {:?} Status changed to Leader", node.id);

                params.watchdog_event_tx.send(LeaderConfirmationEvent::ResetWatchdogCounter).expect("can send LeaderElectedEvent");
                params.leader_initial_heartbeat_tx.send(true).expect("can send leader initial heartbeat");
            },
            LeaderElectionEvent::ResetNodeToFollower(vr) => {
                let mut node = params.protected_node.lock().expect("node lock is poisoned");

                node.set_current_term(vr.term);
                node.status = NodeStatus::Follower;
                node.voted_for_id = None;
                info!("Node {:?} Status changed to Follower", node.id);

                params. watchdog_event_tx.send(LeaderConfirmationEvent::ResetWatchdogCounter).expect("can send LeaderConfirmationEvent");
            },
        }
    }
}

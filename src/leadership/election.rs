use std::sync::{Arc, Mutex};
use crossbeam_channel::{Sender, Receiver};
use std::thread;

use crate::common::{LeaderConfirmationEvent};
use crate::state::{Node, NodeStatus};
use crate::communication::peers::{InProcNodeCommunicator};
use crate::configuration::cluster::{ClusterConfiguration};
use super::peer_notifier;
use crate::operation_log::storage::LogStorage;

pub enum LeaderElectionEvent {
    PromoteNodeToCandidate(ElectionNotice),
    PromoteNodeToLeader(u64),
    ResetNodeToFollower(ElectionNotice),
}

pub struct ElectionNotice {
    pub term: u64,
    pub candidate_id : u64 //TODO introduce options Candidate, Leader
}


pub fn run_leader_election_process<Log: Sync + Send + LogStorage>(protected_node: Arc<Mutex<Node<Log>>>,
                                                                  leader_election_event_tx : Sender<LeaderElectionEvent>,
                                                                  leader_election_event_rx : Receiver<LeaderElectionEvent>,
                                                                  leader_initial_heartbeat_tx : Sender<bool>,
                                                                  watchdog_event_tx : Sender<LeaderConfirmationEvent>,
                                                                  communicator : InProcNodeCommunicator,
                                                                  cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
) {
    loop {
        let event_result = leader_election_event_rx.recv();

        let event = event_result.expect("can receive election event from channel");

        match event {
            LeaderElectionEvent::PromoteNodeToCandidate(vr) => {
                let mut node = protected_node.lock().expect("node lock is not poisoned");

                let node_id = node.id;
                node.voted_for_id = Some(node_id);
                node.current_leader_id = None;
                node.status = NodeStatus::Candidate;
                info!("Node {:?} Status changed to Candidate", node.id);

                let cluster = cluster_configuration.lock().expect("node lock is not poisoned");

                let (peers_copy, quorum_size )=
                    (cluster.get_peers(node_id), cluster.get_quorum_size());


                let communicator_copy = communicator.clone();
                let election_event_tx_copy = leader_election_event_tx.clone();
                let last_entry_index = node.log.get_last_entry_index() as u64;
                let last_entry_term = node.log.get_last_entry_term();
                thread::spawn(move || peer_notifier::notify_peers(vr.term,
                                                                  election_event_tx_copy,
                                                                  node_id,
                                                                  communicator_copy,
                                                                  peers_copy,
                                                                  quorum_size,
                                                                  last_entry_index,
                                                                  last_entry_term));
            },
            LeaderElectionEvent::PromoteNodeToLeader(term) => {
                let mut node = protected_node.lock().expect("node lock is not poisoned");

                node.current_leader_id = Some(node.id);
                node.set_current_term(term);
                node.voted_for_id = None;
                node.status = NodeStatus::Leader;
                info!("Node {:?} Status changed to Leader", node.id);

                watchdog_event_tx.send(LeaderConfirmationEvent::ResetWatchdogCounter).expect("can send LeaderElectedEvent");
                leader_initial_heartbeat_tx.send(true).expect("can send leader initial heartbeat");
            },
            LeaderElectionEvent::ResetNodeToFollower(vr) => {
                let mut node = protected_node.lock().expect("node lock is poisoned");

                node.set_current_term(vr.term);
                node.status = NodeStatus::Follower;
                node.voted_for_id = None;
                info!("Node {:?} Status changed to Follower", node.id);

                watchdog_event_tx.send(LeaderConfirmationEvent::ResetWatchdogCounter).expect("can send LeaderConfirmationEvent");
            },
        }
    }
}

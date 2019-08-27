use std::sync::{Arc, Mutex};
use crossbeam_channel::{Sender, Receiver};
use std::thread;

use crate::common::{LeaderConfirmationEvent};
use crate::state::{Node, NodeStatus};
use crate::communication::peers::{VoteResponse, InProcNodeCommunicator};
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
    pub candidate_id : u64
}


pub fn run_leader_election_process<Log: Sync + Send + LogStorage>(mutex_node: Arc<Mutex<Node<Log>>>,
                                                                  leader_election_event_tx : Sender<LeaderElectionEvent>,
                                                                  leader_election_event_rx : Receiver<LeaderElectionEvent>,
                                                                  response_event_rx : Receiver<VoteResponse>,
                                                                  leader_initial_heartbeat_tx : Sender<bool>,
                                                                  watchdog_event_tx : Sender<LeaderConfirmationEvent>,
                                                                  communicator : InProcNodeCommunicator,
                                                                  cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
) {
    loop {
        let event_result = leader_election_event_rx.recv();

        let event = event_result.expect("can receive election event from channel");

   //     clean_abort_election_channel(&abort_election_rx);

        match event {
            LeaderElectionEvent::PromoteNodeToCandidate(vr) => {
                let mut node = mutex_node.lock().expect("node lock is not poisoned");

                let node_id = node.id;
                node.voted_for_id = Some(node_id);
                node.current_leader_id = None;
                node.status = NodeStatus::Candidate;
                info!("Node {:?} Status changed to Candidate", node.id);

                let (peers_copy, quorum_size )= {
                    let cluster = cluster_configuration.lock().expect("node lock is not poisoned");

                    (cluster.get_peers(node_id), cluster.get_quorum_size())
                };

                let communicator_copy = communicator.clone();
                let election_event_tx_copy = leader_election_event_tx.clone();
                thread::spawn(move || peer_notifier::notify_peers(vr.term,
                                                                  election_event_tx_copy,
                                                                  node_id,
                                                                  communicator_copy,
                                                                  peers_copy,
                                                                  quorum_size));
            },
            LeaderElectionEvent::PromoteNodeToLeader(term) => {
                let mut node = mutex_node.lock().expect("node lock is not poisoned");

                node.current_leader_id = Some(node.id);
                node.current_term = term;
                node.status = NodeStatus::Leader;
                info!("Node {:?} Status changed to Leader", node.id);

                watchdog_event_tx.send(LeaderConfirmationEvent::ResetWatchdogCounter).expect("can send LeaderElectedEvent");
                leader_initial_heartbeat_tx.send(true).expect("can send leader initial heartbeat");
            },
            LeaderElectionEvent::ResetNodeToFollower(vr) => {
                let mut node = mutex_node.lock().expect("node lock is poisoned");

                node.current_term = vr.term;
                node.status = NodeStatus::Follower;
                info!("Node {:?} Status changed to Follower", node.id);

                watchdog_event_tx.send(LeaderConfirmationEvent::ResetWatchdogCounter).expect("can send LeaderConfirmationEvent");
            },
        }
    }
}

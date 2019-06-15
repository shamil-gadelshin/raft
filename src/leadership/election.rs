use std::sync::{Arc, Mutex};
use crossbeam_channel::{Sender, Receiver};
use std::thread;

use super::core::*;
use super::communication::*;
use super::peer_notifier;

pub fn run_leader_election_process(mutex_node: Arc<Mutex<Node>>,
                                   leader_election_event_tx : Sender<LeaderElectionEvent>,
                                   leader_election_event_rx : Receiver<LeaderElectionEvent>,
                                   response_event_rx : Receiver<VoteResponse>,
                                   watchdog_event_tx : Sender<LeaderConfirmationEvent>,
                                   communicator : InProcNodeCommunicator,
                                   peers : Vec<u64>,
                                   quorum_size : u32
) {

    {
        let node = mutex_node.lock().expect("lock is poisoned");

        print_event(format!("Node {:?} started", node.id ));
    }// mutex lock release

    loop {
        let event_result = leader_election_event_rx.recv();

        let event = event_result.expect("receiving from closed channel");

        match event {
            LeaderElectionEvent::PromoteNodeToCandidate(vr) => {
                let mut node = mutex_node.lock().expect("lock is poisoned");

                node.status = NodeStatus::Candidate;
                print_event(format!("Node {:?} Status changed to Candidate", node.id));
                node.current_leader_id = None;
                let event_sender = leader_election_event_tx.clone();

                let node_id = node.id;
                node.voted_for_id = Some(node_id);
                let peers_copy = peers.clone();
                let communicator_copy = communicator.clone();
                let response_rx_copy = response_event_rx.clone();

                //TODO              optional abort channel for notifier

                thread::spawn(move || peer_notifier::notify_peers(vr.term,
                                                                  event_sender,
                                                                  node_id,
                                                                  communicator_copy,
                                                                  response_rx_copy,
                                                                  peers_copy,
                                                                  quorum_size));
            },
            LeaderElectionEvent::PromoteNodeToLeader(term) => {
                let mut node = mutex_node.lock().expect("lock is poisoned");

                node.current_leader_id = Some(node.id);
                //        node.voted_for_id = None;
                node.current_term = term;
                node.status = NodeStatus::Leader;
                print_event(format!("Node {:?} Status changed to Leader", node.id));

                watchdog_event_tx.send(LeaderConfirmationEvent::ResetWatchdogCounter).expect("cannot send LeaderElectedEvent");
            },
            LeaderElectionEvent::ResetNodeToFollower(vr) => {
                let mut node = mutex_node.lock().expect("lock is poisoned");

                node.current_term = vr.term;
                node.current_leader_id = Some(vr.candidate_id);
                node.status = NodeStatus::Follower;
                print_event(format!("Node {:?} Status changed to Follower", node.id));

                watchdog_event_tx.send(LeaderConfirmationEvent::ResetWatchdogCounter).expect("cannot send LeaderElectedEvent");
            },
        }
    }
}

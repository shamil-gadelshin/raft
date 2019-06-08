use super::core::*;
use std::sync::{Arc, Mutex};
use crossbeam_channel::{Sender, Receiver};

pub fn watch_leader_status(mutex_node: Arc<Mutex<Node>>,
                       leadership_event_tx : Sender<LeaderElectionEvent>,
                       watchdog_event_rx : Receiver<LeaderElectedEvent>) {
    loop {
        let timeout = crossbeam_channel::after(random_awaiting_leader_duration_ms());
        select!(
            recv(timeout) -> _  => {},
            recv(watchdog_event_rx) -> _ => {
                let node = mutex_node.lock().expect("lock is poisoned");
                print_event(format!("Node {:?} Received reset watchdog ", node.id));
                continue
            },
            //TODO : notify leadership
        );

        let node = mutex_node.lock().expect("lock is poisoned");

        if let NodeStatus::Follower = node.status {
            print_event(format!("Node {:?} Leader awaiting time elapsed. Starting new election.", node.id));

            let current_leader_id = node.current_leader_id.clone();

            if current_leader_id.is_none() || current_leader_id.unwrap() != node.id {
                let next_term = node.current_term + 1;
                let candidate_promotion = LeaderElectionEvent::PromoteNodeToCandidate(ElectionNotice { term: next_term, candidate_id: node.id });
                leadership_event_tx.send(candidate_promotion).expect("cannot promote to candidate");
            }
        }
    }
}

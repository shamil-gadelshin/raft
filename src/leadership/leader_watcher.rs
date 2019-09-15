use std::time::{Duration};
use rand::Rng;
use std::sync::{Arc, Mutex};

use crossbeam_channel::{Sender, Receiver};

use crate::common::{LeaderConfirmationEvent};
use crate::state::{Node, NodeStatus};
use super::node_leadership_status::{LeaderElectionEvent, ElectionNotice};
use crate::operation_log::LogStorage;
use crate::fsm::Fsm;
use crate::communication::peers::PeerRequestHandler;


pub struct WatchLeaderStatusParams<Log, FsmT, Pc>
    where Log: Sync + Send + LogStorage + 'static, FsmT: Sync + Send + Fsm + 'static, Pc : PeerRequestHandler + Clone {
    pub protected_node: Arc<Mutex<Node<Log, FsmT, Pc>>>,
    pub leader_election_event_tx : Sender<LeaderElectionEvent>,
    pub watchdog_event_rx : Receiver<LeaderConfirmationEvent>,
}


pub fn watch_leader_status<Log, FsmT, Pc>(params : WatchLeaderStatusParams<Log, FsmT, Pc>)
    where Log: Sync + Send + LogStorage + 'static, FsmT: Sync + Send + Fsm + 'static, Pc : PeerRequestHandler + Clone{
    loop {
        let timeout = crossbeam_channel::after(random_awaiting_leader_duration_ms());
        select!(
            recv(timeout) -> _  => {}, //TODO check err
            recv(params.watchdog_event_rx) -> _ => { //TODO check err
                let node = params.protected_node.lock().expect("node lock is not poisoned");
                trace!("Node {:?} Received reset watchdog ", node.id);
                continue
            },
        );

        let node = params.protected_node.lock().expect("node lock is not poisoned");

        if let NodeStatus::Follower = node.status {
            info!("Node {:?} Leader awaiting time elapsed. Starting new election.", node.id);

            let current_leader_id = node.current_leader_id;

            if current_leader_id.is_none() || current_leader_id.unwrap() != node.id {
                let next_term = node.get_next_term();
                let candidate_promotion = LeaderElectionEvent::PromoteNodeToCandidate(ElectionNotice { term: next_term, candidate_id: node.id });
                params.leader_election_event_tx.send(candidate_promotion).expect("can promote to candidate");
            }
        }
    }
}


fn random_awaiting_leader_duration_ms() -> Duration{
    let range_start = 1000;
    let range_stop = 4000;
    let mut rng = rand::thread_rng();

    Duration::from_millis(rng.gen_range(range_start, range_stop))
}
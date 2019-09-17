use std::sync::{Arc, Mutex};

use crossbeam_channel::{Sender, Receiver};

use crate::common::{LeaderConfirmationEvent};
use crate::state::{Node, NodeStatus, NodeStateSaver};
use super::node_leadership_status::{LeaderElectionEvent, ElectionNotice};
use crate::operation_log::OperationLog;
use crate::fsm::FiniteStateMachine;
use crate::communication::peers::PeerRequestHandler;
use crate::configuration::node::ElectionTimer;


pub struct WatchLeaderStatusParams<Log, Fsm, Pc, Et, Ns>
    where Log: OperationLog,
          Fsm: FiniteStateMachine,
          Pc : PeerRequestHandler,
          Et : ElectionTimer,
          Ns : NodeStateSaver{
    pub protected_node: Arc<Mutex<Node<Log,Fsm, Pc, Ns>>>,
    pub leader_election_event_tx : Sender<LeaderElectionEvent>,
    pub watchdog_event_rx : Receiver<LeaderConfirmationEvent>,
    pub election_timer: Et,
}


pub fn watch_leader_status<Log,Fsm, Pc, Et, Ns>(params : WatchLeaderStatusParams<Log, Fsm, Pc, Et, Ns>,
                                                terminate_worker_rx : Receiver<()>)
    where Log: OperationLog,
          Fsm: FiniteStateMachine,
          Pc : PeerRequestHandler,
          Et : ElectionTimer,
          Ns : NodeStateSaver{
    info!("Watch leader expiration status worker started");
    loop {
        let timeout = crossbeam_channel::after(params.election_timer.get_next_elections_timeout());
        select!(
            recv(terminate_worker_rx) -> res  => {
                if res.is_err() {
                    error!("Abnormal exit for watch leader expiration status worker");
                }
                break
            },
            recv(timeout) -> _  => {
                propose_node_election(&params)
            },
            recv(params.watchdog_event_rx) -> watchdog_event_result => {
                if let Err(err) = watchdog_event_result {
                    error!("Invalid result from watchdog_event_rx: {}", err);
                }
                let node = params.protected_node.lock().expect("node lock is not poisoned");
                trace!("Node {} Received reset watchdog ", node.id);
                continue
            },
        );
    }
    info!("Watch leader expiration status worker stopped");
}

fn propose_node_election<Log, Fsm, Pc, Et, Ns>(params: &WatchLeaderStatusParams<Log, Fsm, Pc, Et, Ns>)
    where Log: OperationLog,
          Fsm: FiniteStateMachine,
          Pc : PeerRequestHandler,
          Et : ElectionTimer,
          Ns : NodeStateSaver{
    let node = params.protected_node.lock().expect("node lock is not poisoned");
    if let NodeStatus::Follower = node.status {
        info!("Node {} Leader awaiting time elapsed. Starting new election", node.id);

        let current_leader_id = node.current_leader_id;

        if current_leader_id.is_none() || current_leader_id.unwrap() != node.id {
            let next_term = node.get_next_term();
            let candidate_promotion = LeaderElectionEvent::PromoteNodeToCandidate(ElectionNotice { term: next_term, candidate_id: node.id });
            params.leader_election_event_tx.send(candidate_promotion).expect("can promote to candidate");
        }
    }
}



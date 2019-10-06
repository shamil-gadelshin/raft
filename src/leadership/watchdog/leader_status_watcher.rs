use crossbeam_channel::{Receiver};

use crate::communication::peers::PeerRequestHandler;
use crate::node::state::{NodeStateSaver, NodeStatus, ProtectedNode};
use crate::operation_log::OperationLog;
use crate::rsm::ReplicatedStateMachine;
use crate::{Cluster, ElectionTimer};
use crate::leadership::watchdog::watchdog_handler::ResetLeadershipEventChannelRx;
use crate::leadership::status::{CandidateInfo};
use crate::leadership::status::administrator::RaftElections;

pub struct WatchLeaderStatusParams<Log, Rsm, Pc, Et, Ns, Cl, Rl, Re>
where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Pc: PeerRequestHandler,
    Et: ElectionTimer,
    Ns: NodeStateSaver,
    Cl: Cluster,
    Rl: ResetLeadershipEventChannelRx,
    Re: RaftElections,
{
    pub protected_node: ProtectedNode<Log, Rsm, Pc, Ns, Cl>,
    pub raft_elections_administrator: Re,
    pub watchdog_event_rx: Rl,
    pub election_timer: Et,
}

pub fn watch_leader_status<Log, Rsm, Pc, Et, Ns, Cl, Rl, Re>(
    params: WatchLeaderStatusParams<Log, Rsm, Pc, Et, Ns, Cl, Rl, Re>,
    terminate_worker_rx: Receiver<()>,
) where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Pc: PeerRequestHandler,
    Et: ElectionTimer,
    Ns: NodeStateSaver,
    Cl: Cluster,
    Rl: ResetLeadershipEventChannelRx,
    Re: RaftElections,
{
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
            recv(params.watchdog_event_rx.reset_leadership_watchdog_rx())
                -> watchdog_event_result => {
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

fn propose_node_election<Log, Rsm, Pc, Et, Ns, Cl, Rl, Re>(
    params: &WatchLeaderStatusParams<Log, Rsm, Pc, Et, Ns, Cl, Rl, Re>,
) where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Pc: PeerRequestHandler,
    Et: ElectionTimer,
    Ns: NodeStateSaver,
    Cl: Cluster,
    Rl: ResetLeadershipEventChannelRx,
    Re: RaftElections
{
    let node = params
        .protected_node
        .lock()
        .expect("node lock is not poisoned");
    if let NodeStatus::Follower = node.status {
        info!(
            "Node {} Leader awaiting time elapsed. Starting new election",
            node.id
        );

        let current_leader_id = node.current_leader_id;

        if current_leader_id.is_none() || current_leader_id.unwrap() != node.id {

            params.raft_elections_administrator.promote_node_to_candidate(CandidateInfo {
                term: node.get_next_term(),
                candidate_id: node.id,
            });
        }
    }
}

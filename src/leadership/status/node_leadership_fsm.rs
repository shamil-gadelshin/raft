use crossbeam_channel::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use crate::communication::peers::PeerRequestHandler;
use crate::leadership::election::{start_election, StartElectionParams};
use crate::node::state::{Node, NodeStateSaver, NodeStatus};
use crate::operation_log::OperationLog;
use crate::rsm::ReplicatedStateMachine;
use crate::{common, Cluster};
use crate::leadership::watchdog::watchdog_handler::ResetLeadershipStatusWatchdog;
use crate::leadership::status::LeaderElectionEvent;
use crate::leadership::status::administrator::{RaftElections, RaftElectionsChannelRx};


pub struct ElectionManagerParams<Log, Rsm, Pc, Ns, Cl, Rl, Re>
where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Pc: PeerRequestHandler,
    Ns: NodeStateSaver,
    Cl: Cluster,
    Rl: ResetLeadershipStatusWatchdog,
    Re: RaftElections + RaftElectionsChannelRx
{
    pub protected_node: Arc<Mutex<Node<Log, Rsm, Pc, Ns, Cl>>>,
    pub election_administrator: Re,
    pub leader_initial_heartbeat_tx: Sender<()>,
    pub leadership_status_watchdog_handler: Rl,
    pub peer_communicator: Pc,
    pub cluster_configuration: Cl,
}

pub fn run_node_status_watcher<Log, Rsm, Pc, Ns, Cl, Rl, Re>(
    params: ElectionManagerParams<Log, Rsm, Pc, Ns, Cl, Rl, Re>,
    terminate_worker_rx: Receiver<()>,
) where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Pc: PeerRequestHandler,
    Ns: NodeStateSaver,
    Cl: Cluster,
    Rl: ResetLeadershipStatusWatchdog,
    Re: RaftElections + RaftElectionsChannelRx
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
            recv(params.election_administrator.leader_election_event_rx()) -> event_result => {
                let event = event_result.expect("can receive election event from channel");
                change_node_leadership_state(&params, event);
            }
        );
    }
    info!("Leader election status watcher worker stopped");
}

fn change_node_leadership_state<Log, Rsm, Pc, Ns, Cl, Rl, Re>(
    params: &ElectionManagerParams<Log, Rsm, Pc, Ns, Cl, Rl, Re>,
    event: LeaderElectionEvent,
) where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Pc: PeerRequestHandler,
    Ns: NodeStateSaver,
    Cl: Cluster,
    Rl: ResetLeadershipStatusWatchdog,
    Re: RaftElections + RaftElectionsChannelRx
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

            info!("Node {} Status changed to Candidate for term {}", node.id, vr.term);

            let peers_copy = params.cluster_configuration.get_peers(node_id);
            let quorum_size = params.cluster_configuration.get_quorum_size();

            let params = StartElectionParams {
                node_id,
                actual_current_term: node.get_current_term() - 1,
                next_term: vr.term,
                last_log_index: node.log.get_last_entry_index(),
                last_log_term: node.log.get_last_entry_term(),
                raft_elections_administrator: params.election_administrator.clone(),
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

            info!("Node {} Status changed to Leader for term {}", node.id, term);

            params.leadership_status_watchdog_handler.reset_leadership_status_watchdog();

            params
                .leader_initial_heartbeat_tx
                .send(())
                .expect("can send leader initial heartbeat");
        }
        LeaderElectionEvent::ResetNodeToFollower(info) => {
            let mut node = params.protected_node.lock().expect("node lock is poisoned");

            if let Some(leader_id) = info.leader_id {
                node.current_leader_id = Some(leader_id);
            }

            node.status = NodeStatus::Follower;

            if info.voted_for_id.is_some() {
                node.set_voted_for_id(info.voted_for_id);
            }
            else if node.get_current_term() < info.term {
                node.set_voted_for_id(None);
            }

            node.set_current_term(info.term);

            info!("Node {} Status changed to Follower for term {}", node.id, info.term);

            params.leadership_status_watchdog_handler.reset_leadership_status_watchdog();
        }
    }
}

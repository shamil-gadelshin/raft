use std::sync::{Arc, Mutex};
use crossbeam_channel::{Sender, Receiver};

use crate::common::{LeaderConfirmationEvent};
use crate::state::{Node, NodeStatus};
use crate::communication::peers::{InProcPeerCommunicator, PeerRequestHandler};
use crate::configuration::cluster::{ClusterConfiguration};
use crate::common;
use crate::operation_log::LogStorage;
use crate::fsm::Fsm;
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


pub struct ElectionManagerParams<Log, FsmT,Pc>
    where Log: Sync + Send + LogStorage + 'static, FsmT: Sync + Send + Fsm + 'static, Pc : PeerRequestHandler + Clone {
    pub protected_node: Arc<Mutex<Node<Log, FsmT, Pc>>>,
    pub leader_election_event_tx : Sender<LeaderElectionEvent>,
    pub leader_election_event_rx : Receiver<LeaderElectionEvent>,
    pub leader_initial_heartbeat_tx : Sender<bool>,
    pub watchdog_event_tx : Sender<LeaderConfirmationEvent>,
    pub peer_communicator: InProcPeerCommunicator,
    pub cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
}


pub fn run_node_status_watcher<Log, FsmT, Pc>(params : ElectionManagerParams<Log, FsmT, Pc>)
    where Log: Sync + Send + LogStorage + 'static, FsmT: Sync + Send + Fsm + 'static, Pc : PeerRequestHandler + Clone {
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
//
//fn run_election(node_id: u64,
//                actual_current_term: u64,
//                next_term: u64,
//                last_log_index: u64,
//                last_log_term: u64,
//                leader_election_event_tx : Sender<LeaderElectionEvent>,
//                cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
//                peer_communicator: InProcPeerCommunicator) {
//
//    let cluster = cluster_configuration.lock().expect("node lock is not poisoned");
//
//    let (peers_copy, quorum_size) =
//        (cluster.get_peers(node_id), cluster.get_quorum_size());
//
//    let vote_request = VoteRequest {
//        candidate_id: node_id,
//        term:next_term,
//        last_log_index,
//        last_log_term};
//
//    //TODO spawn a worker
//    thread::spawn(move || election::start_election(
//        actual_current_term,
//        leader_election_event_tx,
//        node_id,
//        peer_communicator,
//         peers_copy,
//        quorum_size,
//        vote_request));
//}
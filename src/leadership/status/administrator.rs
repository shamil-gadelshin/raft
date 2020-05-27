use crate::leadership::status::{CandidateInfo, FollowerInfo, LeaderElectionEvent};
use crossbeam_channel::{Receiver, Sender};

pub trait RaftElections: Clone + Send + 'static {
    fn reset_node_to_follower(&self, info: FollowerInfo);
    fn promote_node_to_leader(&self, term: u64);
    fn promote_node_to_candidate(&self, info: CandidateInfo);
}

pub trait RaftElectionsChannelRx {
    fn leader_election_event_rx(&self) -> &Receiver<LeaderElectionEvent>;
}

#[derive(Debug, Clone)]
pub struct RaftElectionsAdministrator {
    leader_election_tx: Sender<LeaderElectionEvent>,
    leader_election_rx: Receiver<LeaderElectionEvent>,
}

impl RaftElectionsAdministrator {
    pub fn new() -> RaftElectionsAdministrator {
        let (leader_election_tx, leader_election_rx): (
            Sender<LeaderElectionEvent>,
            Receiver<LeaderElectionEvent>,
        ) = crossbeam_channel::unbounded();

        RaftElectionsAdministrator {
            leader_election_tx,
            leader_election_rx,
        }
    }
}

impl RaftElections for RaftElectionsAdministrator {
    fn reset_node_to_follower(&self, info: FollowerInfo) {
        self.leader_election_tx
            .send(LeaderElectionEvent::ResetNodeToFollower(info))
            .expect("can send leader election event");
    }

    fn promote_node_to_leader(&self, term: u64) {
        self.leader_election_tx
            .send(LeaderElectionEvent::PromoteNodeToLeader(term))
            .expect("can send leader election event");
    }

    fn promote_node_to_candidate(&self, info: CandidateInfo) {
        self.leader_election_tx
            .send(LeaderElectionEvent::PromoteNodeToCandidate(info))
            .expect("can send leader election event");
    }
}

impl RaftElectionsChannelRx for RaftElectionsAdministrator {
    fn leader_election_event_rx(&self) -> &Receiver<LeaderElectionEvent> {
        &self.leader_election_rx
    }
}

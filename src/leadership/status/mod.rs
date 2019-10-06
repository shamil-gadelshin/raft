pub mod node_leadership_fsm;
pub mod administrator;

pub enum LeaderElectionEvent {
	PromoteNodeToCandidate(CandidateInfo),
	PromoteNodeToLeader(u64), //term
	ResetNodeToFollower(FollowerInfo),
}

pub struct CandidateInfo {
	pub term: u64,
	pub candidate_id: u64,
}

pub struct FollowerInfo {
	pub term: u64,
	pub leader_id: Option<u64>,
	pub voted_for_id: Option<u64>,
}

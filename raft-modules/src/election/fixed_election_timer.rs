use raft::ElectionTimer;
use std::time::Duration;

pub struct FixedElectionTimer {
	fixed_duration_ms: u64,
}

impl FixedElectionTimer{
	pub fn new(	fixed_duration_ms : u64) -> FixedElectionTimer {
		FixedElectionTimer{ fixed_duration_ms}
	}
}

impl ElectionTimer for FixedElectionTimer {
	fn get_next_elections_timeout(&self) -> Duration {
		Duration::from_millis(self.fixed_duration_ms)
	}
}
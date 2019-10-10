use raft::ElectionTimer;
use std::time::Duration;

/// Provides fixed time duration. For 'guaranteed leadership' tests.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct FixedElectionTimer {
    fixed_duration_ms: u64,
}

impl FixedElectionTimer {
    /// Creates new FixedElectionTimer with fixed duration in milliseconds.
    pub fn new(fixed_duration_ms: u64) -> FixedElectionTimer {
        FixedElectionTimer { fixed_duration_ms }
    }
}

impl ElectionTimer for FixedElectionTimer {
    fn next_elections_timeout(&self) -> Duration {
        Duration::from_millis(self.fixed_duration_ms)
    }
}

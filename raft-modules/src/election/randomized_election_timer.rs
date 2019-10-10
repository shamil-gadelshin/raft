use raft::ElectionTimer;
use rand::Rng;
use std::time::Duration;

/// Provides random time duration within a range.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct RandomizedElectionTimer {
    range_start_ms: u64,
    range_stop_ms: u64,
}

impl RandomizedElectionTimer {
    /// Creates new RandomizedElectionTimer with time range in milliseconds.
    pub fn new(range_start_ms: u64, range_stop_ms: u64) -> RandomizedElectionTimer {
        if range_start_ms > range_stop_ms || range_stop_ms == 0 {
            panic!(
                "Invalid params: range_start_ms : {}, range_stop_ms : {}",
                range_start_ms, range_stop_ms
            )
        }
        RandomizedElectionTimer {
            range_start_ms,
            range_stop_ms,
        }
    }
}

impl ElectionTimer for RandomizedElectionTimer {
    fn next_elections_timeout(&self) -> Duration {
        let mut rng = rand::thread_rng();

        Duration::from_millis(rng.gen_range(self.range_start_ms, self.range_stop_ms))
    }
}

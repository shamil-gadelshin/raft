use std::time::Duration;

pub mod election;
pub mod status;
pub mod watchdog;
pub mod vote_request_processor;

pub enum LeaderConfirmationEvent {
    ResetWatchdogCounter,
}

/// Election timeout calculator. Determines period of time after which the node convert to the
/// candidate after start of the elections.
pub trait ElectionTimer: Send + 'static {
    /// Calculate next elections timeout.
    fn next_elections_timeout(&self) -> Duration;
}

use std::time::Duration;

pub mod election;
pub mod watchdog;
pub mod node_leadership_fsm;
pub mod vote_request_processor;

pub enum LeaderConfirmationEvent {
    ResetWatchdogCounter,
}

pub trait ElectionTimer: Send + 'static {
    fn get_next_elections_timeout(&self) -> Duration;
}

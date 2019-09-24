use std::time::Duration;

pub mod leader_status_watcher;
pub mod election;
pub mod vote_request_processor;
pub mod node_leadership_fsm;

pub enum LeaderConfirmationEvent {
	ResetWatchdogCounter
}


pub trait ElectionTimer: Send + 'static {
	fn get_next_elections_timeout(&self) -> Duration;
}

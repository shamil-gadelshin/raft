use crossbeam_channel::{Sender, Receiver};
use crate::leadership::LeaderConfirmationEvent;

pub trait ResetLeadershipStatusWatchdog : Clone {
	fn reset_leadership_status_watchdog(&self);
}

pub trait ResetLeadershipEventChannelRx {
	fn reset_leadership_watchdog_rx(&self) -> &Receiver<LeaderConfirmationEvent>;
}


#[derive(Debug, Clone)]
pub struct LeadershipStatusWatchdogHandler  {
	reset_leadership_watchdog_tx: Sender<LeaderConfirmationEvent>,
	reset_leadership_watchdog_rx: Receiver<LeaderConfirmationEvent>,
}


impl LeadershipStatusWatchdogHandler{
	pub fn new() -> LeadershipStatusWatchdogHandler {
		let (reset_leadership_watchdog_tx, reset_leadership_watchdog_rx):
			(Sender<LeaderConfirmationEvent>, Receiver<LeaderConfirmationEvent>) =
			crossbeam_channel::unbounded();

		LeadershipStatusWatchdogHandler {
			reset_leadership_watchdog_tx,
			reset_leadership_watchdog_rx,
		}
	}
}

impl ResetLeadershipStatusWatchdog for LeadershipStatusWatchdogHandler {
	fn reset_leadership_status_watchdog(&self) {
		self.reset_leadership_watchdog_tx
			.send(LeaderConfirmationEvent::ResetWatchdogCounter)
			.expect("can send leadership confirmation event");
	}
}

impl ResetLeadershipEventChannelRx for LeadershipStatusWatchdogHandler {
	fn reset_leadership_watchdog_rx(&self) -> &Receiver<LeaderConfirmationEvent> {
		&self.reset_leadership_watchdog_rx
	}
}

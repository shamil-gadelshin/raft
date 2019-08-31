use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

use crossbeam_channel::{Sender, Receiver};

use crate::operation_log::storage::LogStorage;
use crate::state::Node;
use crate::leadership::election::LeaderElectionEvent;
use crate::leadership::leader_watcher::watch_leader_status;
use crate::common::LeaderConfirmationEvent;


pub fn run_thread<Log: Sync + Send + LogStorage + 'static>(protected_node : Arc<Mutex<Node<Log>>>,
														   leader_election_tx : Sender<LeaderElectionEvent>,
														   reset_leadership_watchdog_rx : Receiver<LeaderConfirmationEvent>) -> JoinHandle<()> {
	let check_leader_thread = thread::spawn(move||
		watch_leader_status(protected_node,leader_election_tx, reset_leadership_watchdog_rx
		));

	check_leader_thread
}

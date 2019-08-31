use std::thread::JoinHandle;
use std::sync::{Arc, Mutex};
use std::thread;

use crossbeam_channel::Receiver;

use crate::state::Node;
use crate::operation_log::storage::LogStorage;
use crate::fsm::updater::update_fsm;
use crate::fsm::Fsm;

pub struct FsmUpdaterParams<Log>
where Log: Sync + Send + LogStorage + 'static {
	pub protected_log : Arc<Mutex<Log>>,
	pub protected_fsm : Arc<Mutex<Fsm>>,
	pub update_fsm_rx : Receiver<bool>
}


pub fn run_thread<Log>(params : FsmUpdaterParams<Log>) -> JoinHandle<()>
	where Log: Sync + Send + LogStorage + 'static  {
	let thread = thread::spawn(move||
		update_fsm(params.protected_log, params.protected_fsm, params.update_fsm_rx));

	thread
}
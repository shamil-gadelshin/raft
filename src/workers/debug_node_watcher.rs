use std::thread::{JoinHandle, sleep};
use std::thread;
use std::sync::{Mutex, Arc};

use crate::state::Node;
use crate::operation_log::storage::LogStorage;
use std::time::Duration;
use crate::fsm::Fsm;


pub fn run_thread<Log: Sync + Send + LogStorage + 'static, FsmT:  Sync + Send + Fsm+ 'static>(protected_node: Arc<Mutex<Node<Log, FsmT>>>) -> JoinHandle<()> {
	let debug_thread = thread::spawn(move|| debug_node_status( protected_node));

	debug_thread
}


fn debug_node_status<Log: Sync + LogStorage + 'static, FsmT:  Sync + Send + Fsm>(protected_node: Arc<Mutex<Node<Log, FsmT>>>) {
	sleep(Duration::from_secs(1000));
	loop {
		let _ = protected_node.lock().expect("node lock is not poisoned");
//		trace!("Node {:?}. {:?}", node.id, node);
		sleep(Duration::from_millis(1000));
	}
}
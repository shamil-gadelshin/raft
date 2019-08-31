use std::thread::{JoinHandle, sleep};
use std::thread;
use std::sync::{Mutex, Arc};

use crate::state::Node;
use crate::operation_log::storage::LogStorage;
use std::time::Duration;


pub fn run_thread<Log: Sync + Send + LogStorage + 'static>(protected_node: Arc<Mutex<Node<Log>>>) -> JoinHandle<()> {
	let debug_mutex_clone = protected_node.clone();
	let debug_thread = thread::spawn(move|| debug_node_status( protected_node));

	debug_thread
}


fn debug_node_status<Log: Sync + LogStorage + 'static>(protected_node: Arc<Mutex<Node<Log>>>) {
	sleep(Duration::from_secs(1000));
	loop {
		let node = protected_node.lock().expect("node lock is not poisoned");
//		trace!("Node {:?}. {:?}", node.id, node);
		sleep(Duration::from_millis(1000));
	}
}
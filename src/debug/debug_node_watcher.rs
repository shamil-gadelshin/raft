use std::thread::{JoinHandle, sleep};
use std::thread;
use std::sync::{Mutex, Arc};

use crate::state::Node;
use crate::operation_log::OperationLog;
use std::time::Duration;
use crate::fsm::FiniteStateMachine;


pub fn run_thread<Log:OperationLog, Fsm: FiniteStateMachine>(protected_node: Arc<Mutex<Node<Log, Fsm>>>) -> JoinHandle<()> {
	let debug_thread = thread::spawn(move|| debug_node_status( protected_node));

	debug_thread
}


fn debug_node_status<Log: OperationLog, Fsm:FiniteStateMachine >(protected_node: Arc<Mutex<Node<Log, FsmT>>>) {
	sleep(Duration::from_secs(100000));
	loop {
		let _ = protected_node.lock().expect("node lock is not poisoned");
//		trace!("Node {:?}. {:?}", node.id, node);
		sleep(Duration::from_millis(1000));
	}
}
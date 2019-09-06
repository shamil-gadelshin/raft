use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

use crate::operation_log::storage::LogStorage;
use crate::state::Node;
use crate::configuration::node::NodeConfiguration;
use crate::request_handler::client::process_client_requests;
use crate::fsm::Fsm;


pub fn run_thread<Log : LogStorage + Sync + Send+ 'static, FsmT:  Sync + Send + Fsm+ 'static>(protected_node : Arc<Mutex<Node<Log, FsmT>>>,
														   node_config : &NodeConfiguration) -> JoinHandle<()> {
	let client_communicator = node_config.client_communicator.clone();
	let client_request_handler_thread = thread::spawn(move|| process_client_requests(
		protected_node,
		client_communicator));

	client_request_handler_thread
}


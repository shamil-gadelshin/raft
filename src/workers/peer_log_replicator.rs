use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

use crossbeam_channel::{Sender, Receiver};

use crate::operation_log::LogStorage;
use crate::state::Node;
use crate::configuration::node::NodeConfiguration;
use crate::operation_log::replication::peer_log_replicator::replicate_log_to_peer;
use crate::fsm::Fsm;

pub fn run_thread<Log, FsmT:  Sync + Send + Fsm+ 'static>(protected_node: Arc<Mutex<Node<Log, FsmT>>>,
					   replicate_log_to_peer_rx: Receiver<u64>,
					   replicate_log_to_peer_tx: Sender<u64>,
					   node_config : &NodeConfiguration) -> JoinHandle<()>
	where Log: Sync + Send + LogStorage + 'static {
	let communicator = node_config.peer_communicator.clone();
	let peer_log_replicator_thread = thread::spawn(move|| replicate_log_to_peer(
		protected_node,
		replicate_log_to_peer_rx,
		replicate_log_to_peer_tx,
		communicator));

	peer_log_replicator_thread
}
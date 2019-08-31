use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

use crossbeam_channel::Receiver;

use crate::configuration::node::NodeConfiguration;
use crate::operation_log::storage::LogStorage;
use crate::operation_log::replication::heartbeat_append_entries_sender::send_heartbeat_append_entries;
use crate::state::Node;


pub fn run_thread<Log : Sync + Send + LogStorage + 'static>(protected_node : Arc<Mutex<Node<Log>>>,
															leader_initial_heartbeat_rx : Receiver<bool>,
															node_config : &NodeConfiguration) -> JoinHandle<()> {

	let cluster_configuration = node_config.cluster_configuration.clone();
	let communicator = node_config.peer_communicator.clone();
	let heartbeat_append_entries_thread = thread::spawn(move|| send_heartbeat_append_entries(protected_node,
																							 cluster_configuration,
																							 leader_initial_heartbeat_rx,
																							 communicator));

	heartbeat_append_entries_thread
}

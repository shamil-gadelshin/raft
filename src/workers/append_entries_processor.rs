use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

use crossbeam_channel::{Sender};

use crate::operation_log::storage::LogStorage;
use crate::state::Node;
use crate::leadership::election::LeaderElectionEvent;
use crate::common::LeaderConfirmationEvent;
use crate::configuration::node::NodeConfiguration;
use crate::operation_log::replication::append_entries_processor::append_entries_processor;
use crate::fsm::Fsm;


pub fn run_thread<Log: Sync + Send + LogStorage + 'static, FsmT:  Sync + Send + Fsm+ 'static>(protected_node : Arc<Mutex<Node<Log, FsmT>>>,
														   reset_leadership_watchdog_tx : Sender<LeaderConfirmationEvent>,
														   leader_election_tx : Sender<LeaderElectionEvent>,
														   node_config : &NodeConfiguration) -> JoinHandle<()> {
	let append_entries_request_rx = node_config.peer_communicator.get_append_entries_request_rx(node_config.node_id);
	let append_entries_response_tx = node_config.peer_communicator.get_append_entries_response_tx(node_config.node_id);
	let append_entries_processor_thread = thread::spawn(move|| append_entries_processor(
		protected_node,
		leader_election_tx,
		append_entries_request_rx,
		append_entries_response_tx,
		reset_leadership_watchdog_tx));

	append_entries_processor_thread
}
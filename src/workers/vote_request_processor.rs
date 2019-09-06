use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

use crossbeam_channel::Sender;

use crate::operation_log::LogStorage;
use crate::state::Node;
use crate::leadership::election::LeaderElectionEvent;
use crate::configuration::node::NodeConfiguration;
use crate::leadership::vote_request_processor::vote_request_processor;
use crate::fsm::Fsm;


pub fn run_thread<Log: Sync + Send  + LogStorage + 'static, FsmT:  Sync + Send + Fsm+ 'static>(protected_node : Arc<Mutex<Node<Log, FsmT>>>,
															 leader_election_tx : Sender<LeaderElectionEvent>,
															 node_config : &NodeConfiguration) -> JoinHandle<()> {
	let communicator = node_config.peer_communicator.clone();
	let vote_request_channel_rx = node_config.peer_communicator.get_vote_request_rx(node_config.node_id);
	let vote_request_processor_thread = thread::spawn(move || vote_request_processor(leader_election_tx,
																					 protected_node,
																					 communicator,
																					 vote_request_channel_rx));

	vote_request_processor_thread
}
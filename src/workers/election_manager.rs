use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, Sender};

use crate::operation_log::storage::LogStorage;
use crate::state::Node;
use crate::leadership::election::{LeaderElectionEvent, run_leader_election_process};
use crate::configuration::node::NodeConfiguration;
use crate::common::LeaderConfirmationEvent;
use crate::fsm::Fsm;


pub fn run_thread<Log: Sync + Send + LogStorage + 'static, FsmT:  Sync + Send + Fsm+ 'static>(protected_node : Arc<Mutex<Node<Log, FsmT>>>,
														   node_config : &NodeConfiguration,
														   leader_election_rx : Receiver<LeaderElectionEvent>,
														   leader_election_tx : Sender<LeaderElectionEvent>,
														   leader_initial_heartbeat_tx : Sender<bool>,
														   reset_leadership_watchdog_tx : Sender<LeaderConfirmationEvent>

) -> JoinHandle<()> {
	let vote_response_rx_channel = node_config.peer_communicator.get_vote_response_rx(node_config.node_id);
	let cluster_config = node_config.cluster_configuration.clone();
	let communicator = node_config.peer_communicator.clone();

	let run_thread = thread::spawn(move|| run_leader_election_process(protected_node,
																	  leader_election_tx,
																	  leader_election_rx,
																	  leader_initial_heartbeat_tx,
																	  reset_leadership_watchdog_tx,
																	  communicator,
																	  cluster_config));

	run_thread
}
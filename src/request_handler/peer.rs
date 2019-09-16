use std::sync::{Arc, Mutex};
use std::time::Duration;
use crossbeam_channel::{Sender};

use crate::leadership::node_leadership_status::{LeaderElectionEvent};
use crate::leadership::vote_request_processor::process_vote_request;
use crate::communication::peers::{VoteRequest, AppendEntriesRequest, PeerRequestHandler, PeerRequestChannels};
use crate::state::{Node};
use crate::operation_log::OperationLog;
use crate::fsm::FiniteStateMachine;
use crate::common::LeaderConfirmationEvent;
use crate::operation_log::replication::append_entries_processor::process_append_entries_request;


pub struct PeerRequestHandlerParams<Log, Fsm,Pc>
	where Log: OperationLog,
		  Fsm: FiniteStateMachine,
		  Pc : PeerRequestChannels + PeerRequestHandler{
	pub protected_node: Arc<Mutex<Node<Log, Fsm, Pc>>>,
	pub peer_communicator: Pc,
	pub leader_election_event_tx: Sender<LeaderElectionEvent>,
	pub reset_leadership_watchdog_tx: Sender<LeaderConfirmationEvent>
}

pub fn process_peer_request<Log, Fsm,Pc>(params : PeerRequestHandlerParams<Log, Fsm,Pc>)
	where Log: OperationLog,
		  Fsm: FiniteStateMachine,
		  Pc : PeerRequestChannels + PeerRequestHandler{
	let node_id = {params.protected_node.lock().expect("node lock is not poisoned").id};
	let vote_request_rx = params.peer_communicator.vote_request_rx(node_id).clone();
	let append_entries_request_rx = params.peer_communicator.append_entries_request_rx(node_id);

	loop {
		select!(
			recv(vote_request_rx) -> res => {
				let request = res.expect("can get request from vote_request_rx");

				handle_vote_request(node_id,request,  &params);
			},
			recv(append_entries_request_rx) -> res => {
				let request = res.expect("can get request from append_entries_request_rx");

				handle_append_entries_request(node_id, request,  &params);
			}
		);

	}
}

fn handle_vote_request<Log, Fsm,Pc>(node_id: u64, request : VoteRequest, params : &PeerRequestHandlerParams<Log, Fsm,Pc>)
	where Log: OperationLog,
		  Fsm: FiniteStateMachine,
		  Pc : PeerRequestChannels + PeerRequestHandler{
	info!("Node {:?} Received  vote request {:?}", node_id, request);

	let vote_response = process_vote_request(request,
											 params.protected_node.clone(),
											 params.leader_election_event_tx.clone()
											 );
	let resp_result = {
		trace!("Node {} Sending response {:?}", node_id, vote_response);
		let timeout = Duration::from_secs(1);
		params.peer_communicator.vote_response_tx(node_id).send_timeout(vote_response, timeout)
	};
	info!("Node {:?} voted {:?}", node_id, resp_result);
}


pub fn handle_append_entries_request<Log, Fsm,Pc>(node_id : u64, request : AppendEntriesRequest, params : &PeerRequestHandlerParams<Log, Fsm, Pc>)
	where Log: OperationLog,
		  Fsm: FiniteStateMachine,
		  Pc : PeerRequestChannels + PeerRequestHandler{
	let append_entries_response_tx = params.peer_communicator.append_entries_response_tx(node_id);
	trace!("Node {:?} Received 'Append Entries Request' {:?}", node_id, request);

	let append_entry_response = process_append_entries_request(request, params.protected_node.clone(),
	params.leader_election_event_tx.clone(), params.reset_leadership_watchdog_tx.clone());

	//TODO outer communication timeouts
	let send_result = append_entries_response_tx.send_timeout(append_entry_response, Duration::from_secs(1));
	trace!("Node {:?} AppendEntriesResponse: {:?}", node_id, send_result);
}




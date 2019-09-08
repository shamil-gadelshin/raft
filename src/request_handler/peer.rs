use std::sync::{Arc, Mutex};
use crossbeam_channel::{Sender, Receiver};

use crate::leadership::election::{LeaderElectionEvent};
use crate::leadership::vote_request_processor::process_vote_request;
use crate::communication::peers::{VoteRequest, InProcPeerCommunicator};
use crate::state::{Node};
use crate::operation_log::LogStorage;
use crate::fsm::Fsm;


pub struct PeerRequestHandlerParams<Log, FsmT>
	where Log: Sync + Send + LogStorage + 'static, FsmT: Sync + Send + Fsm + 'static {
	pub protected_node : Arc<Mutex<Node<Log, FsmT>>>,
	pub peer_communicator : InProcPeerCommunicator,
	pub leader_election_event_tx : Sender<LeaderElectionEvent>,
	pub vote_request_event_rx : Receiver<VoteRequest>,
}

pub fn process_peer_request<Log: Sync + Send + LogStorage, FsmT:  Sync + Send + Fsm>(params : PeerRequestHandlerParams<Log, FsmT>)
	where Log: Sync + Send + LogStorage + 'static, FsmT: Sync + Send + Fsm + 'static {
	let vote_request_rx = params.vote_request_event_rx.clone();
	loop {
		let request = vote_request_rx.recv().expect("can get request from request_event_rx");
		let node_id = {
			let node = params.protected_node.lock().expect("node lock is not poisoned");

			node.id
		};

		info!("Node {:?} Received request {:?}", node_id, request);

		let vote_response = process_vote_request(params.protected_node.clone(),
												 params.leader_election_event_tx.clone(),
												 request);
		let resp_result = params.peer_communicator.send_vote_response(request.candidate_id, vote_response);
		info!("Node {:?} Voted {:?}", node_id, resp_result);
	}
}






//
//pub fn process_peer_requests<Log: Sync + Send + LogStorage, FsmT:  Sync + Send + Fsm>(params : PeerRequestHandlerParams<Log,FsmT>) {
//	let add_server_request_rx = params.p.get_add_server_request_rx();
//	let new_data_request_rx = params.client_communicator.get_new_data_request_rx();
//	loop {
//		let process_request_result;
//		select!(
//            recv(add_server_request_rx) -> res => {
//				let request = res.expect("can get add server request");
//				let entry_content = EntryContent::AddServer(AddServerEntryContent { new_server: request.new_server });
//					process_request_result = process_client_request(params.protected_node.clone(),
//					params.client_communicator.get_add_server_response_tx(),
//					entry_content);            },
//            recv(new_data_request_rx) -> res => {
//            	let request = res.expect("can get new_data request");
//				let entry_content = EntryContent::Data(DataEntryContent { data: request.data });
//					process_request_result = process_client_request(params.protected_node.clone(),
//					params.client_communicator.get_new_data_response_tx(),
//					entry_content);
//            },
//        );
//
//		trace!("Client request processed: {:?}", process_request_result)
//	}
//}
//
//

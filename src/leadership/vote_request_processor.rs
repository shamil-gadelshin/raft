use std::sync::{Arc, Mutex};
use crossbeam_channel::{Sender, Receiver};

use super::election::{LeaderElectionEvent, ElectionNotice};
use crate::communication::peers::{VoteRequest, VoteResponse, InProcNodeCommunicator};
use crate::state::{Node};
use crate::operation_log::storage::LogStorage;

pub fn vote_request_processor<Log: Sync + Send + LogStorage>(leader_election_event_tx : Sender<LeaderElectionEvent>,
                                                             protected_node: Arc<Mutex<Node<Log>>>,
                                                             communicator : InProcNodeCommunicator,
                                                             request_event_rx : Receiver<VoteRequest>,
) {

    loop {
        let request = request_event_rx.recv().expect("can get request from request_event_rx");
        let mut node = protected_node.lock().expect("node lock is not poisoned");

        info!("Node {:?} Received request {:?}", node.id, request);
        let mut vote_granted = false;
        let mut response_current_term= node.get_current_term();

        //TODO check log
        if node.get_current_term() <= request.term {
            let is_same_term_and_candidate = node.get_current_term() == request.term
                && node.voted_for_id.is_some()
                && node.voted_for_id.expect("vote_id_result") == request.candidate_id;

            if node.get_current_term() < request.term || is_same_term_and_candidate {
                if node.check_log_for_last_entry(request.last_log_term, request.last_log_index as usize) {
                    vote_granted = true;
                    response_current_term = request.term;

                    let follower_event = ElectionNotice { term: request.term, candidate_id: request.candidate_id };
                    leader_election_event_tx.send(LeaderElectionEvent::ResetNodeToFollower(follower_event)).expect("can send LeaderElectionEvent");
                }
            }
        }
        let vote_response = VoteResponse { vote_granted, peer_id: node.id, term: response_current_term };
        let resp_result = communicator.send_vote_response(request.candidate_id, vote_response);
        info!("Node {:?} Voted {:?}", node.id, resp_result);
    }
}



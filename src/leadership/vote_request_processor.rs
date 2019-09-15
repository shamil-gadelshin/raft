use std::sync::{Arc, Mutex};
use crossbeam_channel::{Sender};

use super::node_leadership_status::{LeaderElectionEvent, ElectionNotice};
use crate::communication::peers::{VoteRequest, VoteResponse, PeerRequestHandler};
use crate::state::{Node};
use crate::operation_log::LogStorage;
use crate::fsm::Fsm;


pub fn process_vote_request<Log, FsmT,Pc>(request: VoteRequest,protected_node : Arc<Mutex<Node<Log, FsmT, Pc>>>,
                                                                                 leader_election_event_tx : Sender<LeaderElectionEvent>) -> VoteResponse
    where Log: Sync + Send + LogStorage + 'static, FsmT: Sync + Send + Fsm + 'static, Pc : PeerRequestHandler + Clone {
    let node = protected_node.lock().expect("node lock is not poisoned");

    let mut vote_granted = false;
    let mut response_current_term = node.get_current_term();

    if node.get_current_term() <= request.term {
        let is_same_term_and_candidate = node.get_current_term() == request.term
            && node.voted_for_id.is_some()
            && node.voted_for_id.expect("vote_id_result") == request.candidate_id;

        if (node.get_current_term() < request.term || is_same_term_and_candidate) &&
            node.check_log_for_last_entry(request.last_log_term, request.last_log_index) {
            vote_granted = true;
            response_current_term = request.term;

            let follower_event = ElectionNotice { term: request.term, candidate_id: request.candidate_id };
            leader_election_event_tx.send(LeaderElectionEvent::ResetNodeToFollower(follower_event)).expect("can send LeaderElectionEvent");
        }
    }

    VoteResponse { vote_granted, peer_id: node.id, term: response_current_term }
}


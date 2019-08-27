use std::sync::{Arc, Mutex};
use crossbeam_channel::{Sender, Receiver};

use super::election::{LeaderElectionEvent, ElectionNotice};
use crate::communication::peers::{VoteRequest, VoteResponse, InProcNodeCommunicator};
use crate::state::{Node};
use crate::operation_log::storage::LogStorage;

pub fn vote_request_processor<Log: Sync + Send + LogStorage>(leader_election_event_tx : Sender<LeaderElectionEvent>,
                                                             mutex_node: Arc<Mutex<Node<Log>>>,
                                                             communicator : InProcNodeCommunicator,
                                                             request_event_rx : Receiver<VoteRequest>,
) {

    loop {
        let request = request_event_rx.recv().expect("can get request from request_event_rx");
        let mut node = mutex_node.lock().expect("node lock is not poisoned");

        info!("Node {:?} Received request {:?}", node.id, request);
        let mut vote_granted = false;

        if node.current_term < request.term {
            vote_granted = true;
            node.voted_for_id = Some(request.candidate_id);
            let follower_event = ElectionNotice { term: request.term, candidate_id: request.candidate_id };
            leader_election_event_tx.send(LeaderElectionEvent::ResetNodeToFollower(follower_event)).expect("can send LeaderElectionEvent");
        }
        communicator.send_vote_response(request.candidate_id, VoteResponse { vote_granted, peer_id: node.id, term: request.term })
    }
}


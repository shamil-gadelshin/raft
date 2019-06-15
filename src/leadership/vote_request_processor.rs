use std::sync::{Arc, Mutex};
use crossbeam_channel::{Sender, Receiver};

use super::election::{LeaderElectionEvent, ElectionNotice};
use crate::communication::*;
use crate::core::*;

pub fn vote_request_processor(  leader_election_event_tx : Sender<LeaderElectionEvent>,
                                mutex_node: Arc<Mutex<Node>>,
                                communicator : InProcNodeCommunicator,
                                request_event_rx : Receiver<VoteRequest>,
) {

    loop {
        let request_result = request_event_rx.recv();
        let request = request_result.unwrap(); //TODO
        let mut node = mutex_node.lock().expect("lock is poisoned");

        print_event(format!("Node {:?} Received request {:?}", node.id, request));
        let mut vote_granted = false;

        if node.current_term < request.term {
            //      if let None = node.voted_for_id {
            vote_granted = true;
            node.voted_for_id = Some(request.candidate_id);
            let follower_event = ElectionNotice { term: request.term, candidate_id: request.candidate_id };
            leader_election_event_tx.send(LeaderElectionEvent::ResetNodeToFollower(follower_event)).expect("cannot send LeaderElectionEvent");
        }
        //  }
        communicator.send_vote_response(request.candidate_id, VoteResponse { vote_granted, peer_id: node.id, term: request.term })
    }
}


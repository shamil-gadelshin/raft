use std::time::Duration;
use crossbeam_channel::{Sender, Receiver};

use super::election::{LeaderElectionEvent, ElectionNotice};
use crate::communication::peers::{VoteRequest, VoteResponse, InProcNodeCommunicator};
use crate::core::*;

pub fn notify_peers(term : u64,
                election_event_tx : Sender<LeaderElectionEvent>,
                node_id : u64,
                communicator : InProcNodeCommunicator,
                vote_response_event_rx : Receiver<VoteResponse>,
                //     abort_election_event_rx : Receiver<LeaderElectedEvent>,
                peers : Vec<u64>,
                quorum_size: u32) {
    let vote_request = VoteRequest { candidate_id: node_id, term };
    let peers_exist = !peers.is_empty();
    for peer_id in peers {
        communicator.send_vote_request(peer_id, vote_request);
    }

    let mut votes = 1;

    if peers_exist {
        let timeout = crossbeam_channel::after(notify_peers_timeout_duration_ms());

        loop {
            select!(
            recv(timeout) -> _ => {
                print_event(format!("Leader election timed out for {:?} ", node_id));
                break;
            },
//            recv(abort_election_event_rx) -> _ => {
//                println!("Leader election received for {:?} ", node_id);
//                return;
//            },
            recv(vote_response_event_rx) -> response => {
                let resp = response.unwrap(); //TODO

                print_event(format!("Node {:?} Receiving response {:?}",node_id,  resp));
                if (resp.term == term) && (resp.vote_granted) {
                    votes += 1;
                }
            }
        );
        }
    }

    if votes >= quorum_size {
        let event_promote_to_leader = LeaderElectionEvent::PromoteNodeToLeader(term);

        election_event_tx.send(event_promote_to_leader).expect("cannot promote to leader");
    } else {
        election_event_tx.send(LeaderElectionEvent::ResetNodeToFollower(ElectionNotice{candidate_id : node_id, term: term }))
            .expect("cannot send LeaderElectionEvent");

    }
}


fn notify_peers_timeout_duration_ms() -> Duration{
    Duration::from_millis(100)
}

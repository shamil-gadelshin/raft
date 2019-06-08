use std::time::Duration;
use crossbeam_channel::{Sender, Receiver};

use super::core::*;
use super::communication::*;

pub fn notify_peers(term : u64,
                election_event_tx : Sender<LeaderElectionEvent>,
                node_id : u64,
                communicator : InProcNodeCommunicator,
                vote_response_event_rx : Receiver<VoteResponse>,
                //     abort_election_event_rx : Receiver<LeaderElectedEvent>,
                peers : Vec<u64>,
                quorum_size: u32) {
    let vote_request = VoteRequest { candidate_id: node_id, term };
    for peer_id in peers {
        communicator.send_vote_request(peer_id, vote_request);
    }

    let timeout = crossbeam_channel::after(notify_peers_timeout_duration_ms());

    let mut votes = 1;
    loop {
        select!(
            recv(timeout) -> _ => {
                println!("Leader election failed for {:?} ", node_id);
                election_event_tx.send(LeaderElectionEvent::ResetNodeToFollower(ElectionNotice{candidate_id : node_id, term}))
                    .expect("cannot send LeaderElectionEvent");
                return;
            },
//            recv(abort_election_event_rx) -> _ => {
//                println!("Leader election received for {:?} ", node_id);
//                return;
//            },
            recv(vote_response_event_rx) -> response => {
                let resp = response.unwrap(); //TODO

                println!("receiving response {:?}", resp);
                if (resp.term == term) && (resp.vote_granted) {
                    votes += 1;

                    if votes >= quorum_size {
                        break;
                    }
                }
            }
        );
    }

    let event_promote_to_leader = LeaderElectionEvent::PromoteNodeToLeader(term);

    election_event_tx.send(event_promote_to_leader).expect("cannot promote to leader");
}


fn notify_peers_timeout_duration_ms() -> Duration{
    Duration::from_millis(1000)
}

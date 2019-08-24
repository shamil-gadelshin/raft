use std::time::Duration;
use std::thread;
use crossbeam_channel::{Sender, Receiver};

use super::election::{LeaderElectionEvent, ElectionNotice};
use crate::communication::peers::{VoteRequest, VoteResponse, InProcNodeCommunicator};
use crate::core::*;

//TODO refactor to DuplexChannel. Watch out for election timeout
pub fn notify_peers(term : u64,
                election_event_tx : Sender<LeaderElectionEvent>,
                node_id : u64,
                communicator : InProcNodeCommunicator,
                //     abort_election_event_rx : Receiver<LeaderElectedEvent>, TODO ?
                peers : Vec<u64>,
                quorum_size: u32) {
    let vote_request = VoteRequest { candidate_id: node_id, term };
    let peers_exist = !peers.is_empty();
    let mut votes = 1;
    let peers_exist = !peers.is_empty();

    for peer_id in peers {
        communicator.send_vote_request(peer_id, vote_request);
    }

    if peers_exist {
        let timeout = crossbeam_channel::after(notify_peers_timeout_duration_ms());

        let (quorum_gathered_tx, quorum_gathered_rx): (Sender<bool>, Receiver<bool>) = crossbeam_channel::unbounded();
        let (terminate_thread_tx, terminate_thread_rx): (Sender<bool>, Receiver<bool>) = crossbeam_channel::unbounded();

        thread::spawn(move || process_votes(node_id, communicator, quorum_gathered_tx, terminate_thread_rx, term, quorum_size));
        loop {
            select!(
            recv(timeout) -> _ => {
                print_event(format!("Leader election timed out for NodeId =  {:?} ", node_id));
                terminate_thread_tx.send(true);
                break;
            },
            recv(quorum_gathered_rx) -> _ => {
                print_event(format!("Leader election - quorum gathered for NodeId = {:?} ", node_id));

                let event_promote_to_leader = LeaderElectionEvent::PromoteNodeToLeader(term);
                election_event_tx.send(event_promote_to_leader).expect("cannot promote to leader");

                return;
            },
//            recv(abort_election_event_rx) -> _ => {
//                println!("Leader election received for {:?} ", node_id);
//                return;
//            },
//            recv(communicator.get_vote_response_channel_rx(node_id)) -> response => {
//                let resp = response.unwrap(); //TODO
//
//                print_event(format!("Node {:?} Receiving response {:?}",node_id,  resp));
//                if (resp.term == term) && (resp.vote_granted) {
//                    votes += 1;
//                }
//            }
        );
        }
    }

    election_event_tx.send(LeaderElectionEvent::ResetNodeToFollower(ElectionNotice{candidate_id : node_id, term: term }))
            .expect("cannot send LeaderElectionEvent");

}


fn process_votes( node_id : u64,
                  communicator : InProcNodeCommunicator,
               //   peers : Vec<u64>,
                  quorum_gathered_tx: Sender<bool>,
                  terminate_thread_rx: Receiver<bool>,
                  term : u64,
                  quorum_size: u32) {
    let mut votes = 1;

//    thread::sleep(Duration::from_millis(2000));
    loop {
        select!(
            recv(terminate_thread_rx) -> _ => {
               return;
            },
            recv(communicator.get_vote_response_channel_rx(node_id)) -> response => {
                let resp = response.unwrap(); //TODO

                print_event(format!("Node {:?} Receiving response {:?}",node_id,  resp));
                if (resp.term == term) && (resp.vote_granted) {
                    votes += 1;
                }

                if votes >= quorum_size {
                    quorum_gathered_tx.send(true);
                    return;
                }
             }
        );
    }
}

fn notify_peers_timeout_duration_ms() -> Duration{
    Duration::from_millis(100)
}

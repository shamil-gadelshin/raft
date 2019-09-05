use std::time::Duration;
use std::thread;
use crossbeam_channel::{Sender, Receiver};

use super::election::{LeaderElectionEvent, ElectionNotice};
use crate::communication::peers::{VoteRequest, InProcNodeCommunicator};

//TODO refactor to DuplexChannel.send_request. Watch out for election timeout
//TODO refactor to generic peer_notifier
pub fn notify_peers(term : u64,
                    election_event_tx : Sender<LeaderElectionEvent>,
                    node_id : u64,
                    communicator : InProcNodeCommunicator,
                    peers : Vec<u64>,
                    quorum_size: u32,
                    last_entry_index : u64,
                    last_entry_term : u64) {
    let vote_request = VoteRequest { candidate_id: node_id, term, last_log_index: last_entry_index, last_log_term: last_entry_term };
    let peers_exist = !peers.is_empty();

    for peer_id in peers {
        communicator.send_vote_request(peer_id, vote_request);
    }

    if peers_exist {
        let timeout = crossbeam_channel::after(notify_peers_timeout_duration_ms());

        let (quorum_gathered_tx, quorum_gathered_rx): (Sender<bool>, Receiver<bool>) = crossbeam_channel::unbounded();
        let (terminate_thread_tx, terminate_thread_rx): (Sender<bool>, Receiver<bool>) = crossbeam_channel::unbounded();

        thread::spawn(move || process_votes(node_id, communicator, quorum_gathered_tx, terminate_thread_rx, term, quorum_size));
        select!(
            recv(timeout) -> _ => {
                info!("Leader election timed out for NodeId =  {:?} ", node_id);
                terminate_thread_tx.send(true).expect("can send to terminate_thread_tx");

                election_event_tx.send(LeaderElectionEvent::ResetNodeToFollower(ElectionNotice{candidate_id : node_id, term }))
                    .expect("can send LeaderElectionEvent");
            },
            recv(quorum_gathered_rx) -> _ => {
                info!("Leader election - quorum ({:?}) gathered for NodeId = {:?} ",quorum_size, node_id);

                let event_promote_to_leader = LeaderElectionEvent::PromoteNodeToLeader(term);
                election_event_tx.send(event_promote_to_leader).expect("can promote to leader");
            },
        );
    }
}


fn process_votes( node_id : u64,
                  communicator : InProcNodeCommunicator,
                  quorum_gathered_tx: Sender<bool>,
                  terminate_thread_rx: Receiver<bool>,
                  term : u64,
                  quorum_size: u32) {
    let mut votes = 1;

    loop {
        select!(
            recv(terminate_thread_rx) -> _ => {
               return;
            },
            recv(communicator.get_vote_response_rx(node_id)) -> response => {
                let resp = response.expect("can receive from communicator.get_vote_response_rx(node_id)");

                trace!("Node {:?} Receiving response {:?}",node_id,  resp);
                if (resp.term == term) && (resp.vote_granted) {
                    votes += 1;
                }

                if votes >= quorum_size {
                    quorum_gathered_tx.send(true).expect("can send to quorum_gathered_tx");
                    return;
                }
             }
        );
    }
}

fn notify_peers_timeout_duration_ms() -> Duration{
    Duration::from_millis(100)
}

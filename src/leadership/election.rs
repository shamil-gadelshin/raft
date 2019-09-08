use std::time::Duration;
use std::thread;
use crossbeam_channel::{Sender, Receiver};

use super::node_leadership_status::{LeaderElectionEvent, ElectionNotice};
use crate::communication::peers::{VoteRequest, InProcPeerCommunicator};
use std::error::Error;


pub struct StartElectionParams {
	pub node_id: u64,
	pub actual_current_term: u64,
	pub next_term: u64,
	pub last_log_index: u64,
	pub last_log_term: u64,
	pub leader_election_event_tx: Sender<LeaderElectionEvent>,
	pub peers: Vec<u64>,
	pub quorum_size: u32,
	pub peer_communicator: InProcPeerCommunicator
}


//TODO refactor to DuplexChannel.send_request. Watch out for election timeout
//TODO refactor to generic peer_notifier
pub fn start_election(params : StartElectionParams) {
	let vote_request = VoteRequest {
		candidate_id: params.node_id,
		term:params.next_term,
		last_log_index:params.last_log_index,
		last_log_term: params.last_log_term};

    let peers_exist = !params.peers.is_empty();

    for peer_id in params.peers.clone() {
        let send_vote_result = params.peer_communicator.send_vote_request(peer_id, vote_request);
		if let Err(err) = send_vote_result {
			error!("Vote request for Node {} failed:{}", peer_id, err.description());
		}
    }

    if peers_exist {
        let timeout = crossbeam_channel::after(notify_peers_timeout_duration_ms());

        let (quorum_gathered_tx, quorum_gathered_rx): (Sender<bool>, Receiver<bool>) = crossbeam_channel::unbounded();
        let (terminate_thread_tx, terminate_thread_rx): (Sender<bool>, Receiver<bool>) = crossbeam_channel::unbounded();

		let (node_id, peer_communicator, next_term, quorum_size) = {(params.node_id, params.peer_communicator.clone(), params.next_term, params.quorum_size)};
        thread::spawn( move || process_votes(node_id, peer_communicator.clone(), quorum_gathered_tx, terminate_thread_rx, next_term, quorum_size));
        select!(
            recv(timeout) -> _ => {
                info!("Leader election timed out for NodeId =  {:?} ", params.node_id);
                terminate_thread_tx.send(true).expect("can send to terminate_thread_tx");

                params.leader_election_event_tx.send(LeaderElectionEvent::ResetNodeToFollower(ElectionNotice{candidate_id : params.node_id, term: params.actual_current_term }))
                    .expect("can send LeaderElectionEvent");
            },
            recv(quorum_gathered_rx) -> _ => {
                info!("Leader election - quorum ({:?}) gathered for NodeId = {:?} ",params.quorum_size, params.node_id);

                let event_promote_to_leader = LeaderElectionEvent::PromoteNodeToLeader(vote_request.term);
                params.leader_election_event_tx.send(event_promote_to_leader).expect("can promote to leader");
            },
        );
    }
}


fn process_votes(node_id : u64,
				 communicator : InProcPeerCommunicator,
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

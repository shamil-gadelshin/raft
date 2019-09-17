use crossbeam_channel::{Sender};

use super::node_leadership_status::{LeaderElectionEvent, ElectionNotice};
use crate::communication::peers::{VoteRequest, PeerRequestHandler};
use crate::common::peer_notifier::notify_peers;


pub struct StartElectionParams<Pc : PeerRequestHandler> {
	pub node_id: u64,
	pub actual_current_term: u64,
	pub next_term: u64,
	pub last_log_index: u64,
	pub last_log_term: u64,
	pub leader_election_event_tx: Sender<LeaderElectionEvent>,
	pub peers: Vec<u64>,
	pub quorum_size: u32,
	pub peer_communicator: Pc
}


//TODO Watch out for election timeout
pub fn start_election<Pc : PeerRequestHandler + Clone>(params : StartElectionParams<Pc>) {
	let vote_request = VoteRequest {
		candidate_id: params.node_id,
		term: params.next_term,
		last_log_index: params.last_log_index,
		last_log_term: params.last_log_term
	};

	let peers_exist = !params.peers.is_empty();

	if !peers_exist {
		warn!("Election with no peers");
		return;
	}

	let peer_communicator = params.peer_communicator.clone();
	let requester = |dest_node_id: u64, req: VoteRequest| {
		let resp_result = peer_communicator.send_vote_request(dest_node_id, req);
		let resp = resp_result.expect("can get append_entries response"); //TODO check timeout

		trace!("Destination Node {} vote requested. Result={}", dest_node_id, resp.vote_granted);
		Ok(resp)
	};

	let notify_peers_result = notify_peers(vote_request, params.node_id, params.peers, Some(params.quorum_size), requester);

	match notify_peers_result {
		Ok(won_election) => {
			let election_event;
			if won_election {
				info!("Leader election - quorum ({}) gathered for NodeId = {} ", params.quorum_size, params.node_id);
				election_event = LeaderElectionEvent::PromoteNodeToLeader(vote_request.term);
			} else {
				info!("Leader election failed for Node {} ", params.node_id);
				election_event = LeaderElectionEvent::ResetNodeToFollower(ElectionNotice { candidate_id: params.node_id, term: params.actual_current_term })
			}
			params.leader_election_event_tx.send(election_event).expect("can promote to leader");
		},
		Err(err) => {
			error!("Leader election failed with errors for Node {}:{}", params.node_id, err.description());
		}
	};
}

use crate::common::peer_consensus_requester::request_peer_consensus;
use crate::communication::peers::{PeerRequestHandler, VoteRequest};
use crate::errors;
use crate::leadership::status::{FollowerInfo};
use crate::leadership::status::administrator::RaftElections;

pub struct StartElectionParams<Pc, Re>
where Pc: PeerRequestHandler,
      Re: RaftElections {
    pub node_id: u64,
    pub actual_current_term: u64,
    pub next_term: u64,
    pub last_log_index: u64,
    pub last_log_term: u64,
    pub raft_elections_administrator: Re,
    pub peers: Vec<u64>,
    pub quorum_size: u32,
    pub peer_communicator: Pc,
}

pub fn start_election<Pc, Re> (params: StartElectionParams<Pc, Re>)
where Pc: PeerRequestHandler + Clone,
      Re: RaftElections {
    let vote_request = VoteRequest {
        candidate_id: params.node_id,
        term: params.next_term,
        last_log_index: params.last_log_index,
        last_log_term: params.last_log_term,
    };

    let peers_exist = !params.peers.is_empty();

    if !peers_exist {
        warn!("Election with no peers");

        params.raft_elections_administrator.promote_node_to_leader(params.next_term);
        return;
    }

    let peer_communicator = params.peer_communicator.clone();
    let requester = |dest_node_id: u64, req: VoteRequest| {
        let resp_result = peer_communicator.send_vote_request(dest_node_id, req);
        match resp_result {
            Ok(resp) => {
                trace!("Destination Node {} vote requested. Result={}",
                    dest_node_id, resp.vote_granted);
                Ok(resp)
            }
            Err(err) => {
                let msg = format!("Destination Node {} vote request failed:{}",
                    dest_node_id, err);
                error!("{}", msg);

                errors::new_err("Cannot get vote from peer".to_string(), msg)
            }
        }
    };

    let notify_peers_result = request_peer_consensus(
        vote_request,
        params.node_id,
        params.peers,
        Some(params.quorum_size),
        requester,
    );

    match notify_peers_result {
        Ok(won_election) => {
            if won_election {
                info!("Leader election - quorum ({}) gathered for Node {} ",
                    params.quorum_size, params.node_id);

                params.raft_elections_administrator.promote_node_to_leader(vote_request.term);
            } else {
                info!("Leader election failed for Node {} ", params.node_id);
                params.raft_elections_administrator.reset_node_to_follower(
                    FollowerInfo {
                        term: params.actual_current_term,
                        leader_id: None,
                        voted_for_id: Some(params.node_id)
                    });
            }

        }
        Err(err) => {
            error!("Leader election failed with errors for Node {}:{}", params.node_id, err);
        }
    };
}

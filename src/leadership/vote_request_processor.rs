use crate::communication::peers::{PeerRequestHandler, VoteRequest, VoteResponse};
use crate::leadership::status::administrator::RaftElections;
use crate::leadership::status::FollowerInfo;
use crate::node::state::{NodeStateSaver, ProtectedNode};
use crate::operation_log::OperationLog;
use crate::rsm::ReplicatedStateMachine;
use crate::Cluster;

pub fn process_vote_request<Log, Rsm, Pc, Ns, Cl, Re>(
    request: VoteRequest,
    protected_node: ProtectedNode<Log, Rsm, Pc, Ns, Cl>,
    raft_elections_administrator: Re,
) -> VoteResponse
where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Pc: PeerRequestHandler,
    Ns: NodeStateSaver,
    Cl: Cluster,
    Re: RaftElections,
{
    let node = protected_node.lock().expect("node lock is not poisoned");

    let mut vote_granted = false;
    let mut response_current_term = node.current_term();

    if node.current_term() <= request.term {
        let is_same_term_and_candidate = node.current_term() == request.term
            && node.voted_for_id().is_some()
            && node.voted_for_id().expect("vote_id_result") == request.candidate_id;

        if (node.current_term() < request.term || is_same_term_and_candidate)
            && node.check_candidate_last_log_entry(request.last_log_term, request.last_log_index)
        {
            vote_granted = true;
            response_current_term = request.term;

            raft_elections_administrator.reset_node_to_follower(FollowerInfo {
                term: request.term,
                leader_id: None,
                voted_for_id: Some(request.candidate_id),
            });
        }
    }

    VoteResponse {
        vote_granted,
        peer_id: node.id,
        term: response_current_term,
    }
}

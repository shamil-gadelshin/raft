
pub trait RsmFacet {}

pub trait OperationLogFacet {}

pub trait NodeStateFacet {}

//
//PromoteNodeToCandidate(CandidateInfo),
//PromoteNodeToLeader(u64), //term
//ResetNodeToFollower(FollowerInfo),





pub trait ElectionFacet {
    fn reset_to_follower();
    //..
    //reset watch_dog??
}


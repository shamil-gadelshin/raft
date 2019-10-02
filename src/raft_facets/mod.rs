pub trait RsmFacet {}
pub trait ElectionFacet {
    fn reset_to_follower();
    //..
    //reset watch_dog??
}
pub trait OperationLogFacet {}

pub trait NodeStateFacet {}

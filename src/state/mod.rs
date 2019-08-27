use crate::operation_log::storage::LogStorage;

#[derive(Debug, Clone)]
pub struct Node<Log: LogStorage + Sized + Sync> {
    pub id : u64, //TODO pass node_id as copy to decrease mutex lock count
    pub current_term: u64,
    pub current_leader_id: Option<u64>,
    pub voted_for_id: Option<u64>,
    pub status : NodeStatus,
    pub log : Log,
}


impl <Log: Sized + Sync + LogStorage> Node<Log> {
    pub fn get_last_entry_index(&self) -> u64{
        self.log.get_last_entry_index()
    }
    pub fn get_last_entry_term(&self) -> u64{
        self.log.get_last_entry_term()
    }

    //TODO support the fsm
    pub fn get_last_applied_index(&self) -> u64{
        self.log.get_last_entry_index()
    }
}


#[derive(Copy, Clone, Debug)]
pub enum NodeStatus {
    Follower,
    Candidate,
    Leader
}

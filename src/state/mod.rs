use crate::operation_log::storage::{LogStorage};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::configuration::cluster::ClusterConfiguration;
use crate::communication::peers::{InProcNodeCommunicator, AppendEntriesRequest};
use crate::common::{LogEntry,EntryContent};
use crate::communication::peer_notifier::notify_peers;
use crate::fsm::Fsm;

#[derive(Debug, Clone)]
//TODO persist state
//TODO decompose GOD object
//TODO decompose to Node & NodeState or extract get_peers() from cluster_config
pub struct Node<Log: LogStorage + Sized + Sync> {
    pub id : u64, //TODO pass node_id as copy to decrease mutex lock count
    pub current_term: u64,
    pub current_leader_id: Option<u64>,
    pub voted_for_id: Option<u64>,
    pub status : NodeStatus,
    pub next_index : HashMap<u64, u64>,
    pub match_index : HashMap<u64, u64>,
    pub log : Log,
    pub fsm : Fsm,
    pub communicator : InProcNodeCommunicator,
    pub cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
}


#[derive(Copy, Clone, Debug)]
pub enum NodeStatus {
    Follower,
    Candidate,
    Leader
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


    pub fn append_entry_to_log(&mut self, entry : LogEntry ){
        self.log.append_entry(entry.clone());
        self.fsm.apply_entry(entry);
    }


    //TODO remove entry.clone() !!
    //TODO check result
    pub fn append_content_to_log(&mut self, content : EntryContent ) -> Result<(), &'static str> {
        let entry = self.log.append_content(self.current_term, content);
        let send_result = self.send_append_entries(entry.clone());

        if !send_result.is_ok() {
            return send_result;
        }

        //TODO gather quorum
        self.fsm.apply_entry(entry);

        Ok(())
    }

    fn send_append_entries(&self, entry : LogEntry) -> Result<(), &'static str>{
        if let NodeStatus::Leader = self.status {
            let peers_list_copy =  {
                let cluster = self.cluster_configuration.lock()
                    .expect("cluster lock is not poisoned");

                cluster.get_peers(self.id)
            };

            let entries = vec![entry];

            let append_entries_request =  AppendEntriesRequest {
                term: self.current_term,
                leader_id: self.id,
                prev_log_term : self.get_last_entry_term(),
                prev_log_index : self.get_last_entry_index(),
                leader_commit : 0, //TODO support fsm
                entries
            };

            trace!("Node {:?} Send 'empty Append Entries Request(heartbeat)'.", self.id);

            let requester = |dest_node_id: u64, req: AppendEntriesRequest| self.communicator.send_append_entries_request(dest_node_id, req);

            return notify_peers(append_entries_request, self.id,peers_list_copy, None, requester);
        }
        Err("Not a leader")
    }
}



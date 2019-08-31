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

pub enum AppendEntriesRequestType {
    Heartbeat,
    NewEntry(LogEntry),
    UpdateNode(u64)
}

impl <Log: Sized + Sync + LogStorage> Node<Log> {
    pub fn get_last_entry_index(&self) -> usize{
        self.log.get_last_entry_index()
    }
    pub fn get_last_entry_term(&self) -> u64{
        self.log.get_last_entry_term()
    }

    pub fn get_last_applied_index(&self) -> usize{
        self.fsm.get_last_applied_entry_index()
    }

    pub fn append_entry_to_log(&mut self, entry : LogEntry ){
        self.log.append_entry(entry.clone());

        self.fsm.apply_entry(entry);
        //fsm.check_updates();
    }


    //TODO check result
    pub fn append_content_to_log(&mut self, content : EntryContent ) -> Result<(), &'static str> {
        let entry = self.log.append_content(self.current_term, content);
        let send_result = self.send_append_entries(entry.clone());

        if !send_result.is_ok() {
            return send_result;
        }

        //fsm.check_updates();
        self.fsm.apply_entry(entry);

        Ok(())
    }

    pub fn create_append_entry_request(&self, request_type : AppendEntriesRequestType) -> AppendEntriesRequest {
        let entries = self.get_log_entries(request_type);
        let append_entry_request = AppendEntriesRequest {
            term: self.current_term,
            leader_id: self.id,
            prev_log_term: self.get_last_entry_term(),
            prev_log_index: self.get_last_entry_index() as u64,
            leader_commit: self.fsm.get_last_applied_entry_index() as u64,
            entries
        };

        append_entry_request
    }

    fn get_log_entries(&self, request_type : AppendEntriesRequestType) -> Vec<LogEntry> {
        match request_type{
            AppendEntriesRequestType::Heartbeat => {
                Vec::new() //empty AppendEntriesRequest - heartbeat
            },
            AppendEntriesRequestType::NewEntry(entry) => {
                let entries = vec![entry];
                entries
            },
            AppendEntriesRequestType::UpdateNode(id) => {
                let entries = Vec::new();
                entries
            }
        }
    }

    fn send_append_entries(&self, entry : LogEntry) -> Result<(), &'static str>{
        if let NodeStatus::Leader = self.status {
            let (peers_list_copy, quorum_size) =  {
                let cluster = self.cluster_configuration.lock()
                    .expect("cluster lock is not poisoned");

                (cluster.get_peers(self.id), cluster.get_quorum_size())
            };

            let append_entries_request =  self.create_append_entry_request(AppendEntriesRequestType::NewEntry(entry));

            trace!("Node {:?} Send 'empty Append Entries Request(heartbeat)'.", self.id);

            let requester = |dest_node_id: u64, req: AppendEntriesRequest| self.communicator.send_append_entries_request(dest_node_id, req);

            return notify_peers(append_entries_request, self.id,peers_list_copy, Some(quorum_size), requester);
        }
        Err("Not a leader")
    }
}



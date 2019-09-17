use std::error::Error;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crossbeam_channel::{Sender};

use crate::configuration::cluster::ClusterConfiguration;
use crate::communication::peers::{AppendEntriesRequest, PeerRequestHandler};
use crate::common::{LogEntry,EntryContent};
use crate::common::peer_notifier::notify_peers;
use crate::fsm::{FiniteStateMachine};
use crate::operation_log::{OperationLog};
use crate::errors;
use crate::errors::new_err;


#[derive(Debug, Clone)]
//TODO decompose GOD object
//TODO decompose to Node & NodeState or extract get_peers() from cluster_config
pub struct Node<Log,Fsm,Pc, Ns>
where Log: OperationLog,
      Fsm: FiniteStateMachine,
      Pc : PeerRequestHandler,
      Ns : NodeStateSaver{
    pub id : u64,
    current_term: u64,
    pub current_leader_id: Option<u64>,
    voted_for_id: Option<u64>,
    pub status : NodeStatus,
    next_index : HashMap<u64, u64>,
    match_index : HashMap<u64, u64>, //TODO support match_index
    commit_index: u64,
    pub log : Log,
    pub fsm : Fsm,
    communicator : Pc,
    cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
    replicate_log_to_peer_tx: Sender<u64> ,//TODO split god object
    commit_index_updated_tx : Sender<u64>,
    state_saver: Ns
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NodeStatus {
    Follower,
    Candidate,
    Leader
}

#[derive(Debug)]
pub struct NodeState {
    pub node_id: u64,
    pub current_term: u64,
    pub vote_for_id: Option<u64>,
}

pub trait NodeStateSaver : Send + 'static{
    fn save_node_state(&self, state : NodeState) -> Result<(), Box<Error>>;
}

pub enum AppendEntriesRequestType {
    Heartbeat,
    NewEntry(LogEntry),
    UpdateNode(u64)
}

//TODO refactor to node_config
impl <Log, Fsm,Pc, Ns> Node<Log, Fsm,Pc, Ns>
where Log: OperationLog,
      Fsm: FiniteStateMachine,
      Pc : PeerRequestHandler,
      Ns : NodeStateSaver{
    pub fn new(id: u64,
               current_term: u64,
               voted_for_id: Option<u64>,
               status: NodeStatus,
               log: Log,
               fsm: Fsm,
               communicator: Pc,
               cluster_configuration: Arc<Mutex<ClusterConfiguration>>,
               replicate_log_to_peer_tx: Sender<u64>,
               commit_index_updated_tx: Sender<u64>,
               state_saver : Ns) -> Node<Log, Fsm, Pc, Ns> {
        Node {
            id,
            current_term,
            current_leader_id: None,
            voted_for_id,
            status,
            next_index: HashMap::new(),
            match_index: HashMap::new(),
            commit_index: 0,
            log,
            fsm,
            communicator,
            cluster_configuration,
            replicate_log_to_peer_tx,
            commit_index_updated_tx,
            state_saver
        }
    }

    pub fn get_current_term(&self) -> u64 {
        if self.status == NodeStatus::Candidate {
            self.current_term + 1
        } else {
            self.current_term
        }
    }

    pub fn set_commit_index(&mut self, new_commit_index: u64) {
        self.commit_index = new_commit_index;
        self.commit_index_updated_tx.send(new_commit_index).expect("can send updated commit_index")
    }

    pub fn get_commit_index(&self) -> u64 {
        self.commit_index
    }

    pub fn get_next_term(&self) -> u64 {
        self.current_term + 1
    }
    pub fn set_current_term(&mut self, new_term: u64) {
        self.current_term = new_term;
        self.save_node_state();
    }

    pub fn get_voted_for_id(&self) -> Option<u64> {
        self.voted_for_id
    }
    pub fn set_voted_for_id(&mut self, voted_for_id: Option<u64>) {
        self.voted_for_id = voted_for_id;
        self.save_node_state();
    }

    fn save_node_state(&self){
        let state = NodeState{
            node_id: self.id,
            current_term: self.current_term,
            vote_for_id: self.voted_for_id
        };

        let result = self.state_saver.save_node_state(state);

        if let Err(err) = result {
            let msg = format!("Node state save failed:{}", err.description());

            error!("{}", msg);
        }
    }

    pub fn get_next_index(&self, peer_id: u64) -> u64 {
        if !self.next_index.contains_key(&peer_id) {
            self.log.get_last_entry_index() as u64
        } else {
            self.next_index[&peer_id]
        }
    }

    pub fn set_next_index(&mut self, peer_id: u64, new_next_index: u64) {
        let next_index_entry = self.next_index.entry(peer_id)
            .or_insert(new_next_index);

        *next_index_entry = new_next_index;
    }


    ///Gets entry by index & compares terms.
    /// Special case index=0, term=0 returns true
    pub fn check_log_for_previous_entry(&self, prev_log_term: u64, prev_log_index: u64) -> bool {
        if prev_log_term == 0 && prev_log_index == 0 {
            return true
        }
        let entry_result = self.log.get_entry(prev_log_index);

        if let Some(entry) = entry_result {
            return entry.term == prev_log_term;
        }

        false
    }

    //TODO comment
    pub fn check_log_for_last_entry(&self, log_term: u64, log_index: u64) -> bool {
        if self.log.get_last_entry_term() > log_term {
            return false
        }
        if self.log.get_last_entry_term() < log_term {
            return true
        }

        if self.log.get_last_entry_index() >= log_index {
            return true
        }
        false
    }

    pub fn append_entry_to_log(&mut self, entry: LogEntry) -> Result<(), Box<Error>> {
        let entry_index = entry.index;

        if self.log.get_last_entry_index() < entry_index {
            let log_append_result = self.log.append_entry(entry.clone()); //TODO error handling
            if let Err(err) = log_append_result {
                return errors::new_err(format!("cannot append entry to log, index = {}", entry_index), Some(err));
            }
        }
        Ok(())
    }


    pub fn append_content_to_log(&mut self, content: EntryContent) -> Result<(), Box<Error>> {
        let entry = self.log.create_next_entry(self.get_current_term(), content);

        let send_result = self.send_append_entries(entry.clone());
        if let Err(err) = send_result {
            let msg = format!("Entry replication failed:{}", err.description());

            error!("{}", msg);
            return new_err(msg, Some(err));
        }

        let add_to_entry_result = self.append_entry_to_log(entry.clone());
        if let Err(err) = add_to_entry_result {
            let msg = format!("Append entry failed:{}", err.description());

            error!("{}", msg);
            return new_err(msg, Some(err));
        }

        self.set_commit_index(entry.index);

        Ok(())
    }

    pub fn create_append_entry_request(&self, request_type : AppendEntriesRequestType) -> AppendEntriesRequest {
        let entries = self.get_log_entries(request_type);

        let (prev_log_term, prev_log_index) = self.get_prev_term_index(&entries);

        AppendEntriesRequest {
            term: self.get_current_term(),
            leader_id: self.id,
            prev_log_term,
            prev_log_index: prev_log_index as u64,
            leader_commit: self.commit_index,
            entries
        }
    }

    fn get_prev_term_index(&self, entries: &Vec<LogEntry>) -> (u64, u64) {
        let (mut prev_log_term, mut prev_log_index) = (0, 0);
        if !entries.is_empty() {
            let new_entry = &entries[0];
            let new_entry_index = new_entry.index;
            if new_entry_index > 1 {
                let prev_entry_index = new_entry_index - 1;
                let prev_entry = self.log.get_entry(prev_entry_index)
                    .unwrap_or_else(|| panic!("entry exist, index =  {}", prev_entry_index));

                prev_log_term = prev_entry.term;
                prev_log_index = prev_entry.index;
            }
        } else if entries.is_empty() {
            let last_index = self.log.get_last_entry_index();
            if last_index > 1 {
                let last_entry = self.log.get_entry(last_index).expect("valid last entry");

                prev_log_term = last_entry.term;
                prev_log_index = last_entry.index;
            }
        };
        (prev_log_term, prev_log_index)
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
            AppendEntriesRequestType::UpdateNode(peer_id) => {
                let next_index = self.get_next_index(peer_id);
                let last_index = self.log.get_last_entry_index();

                let mut entries = Vec::new();
                for idx in  next_index..=last_index {
                    let entry = self.log.get_entry(idx).expect("valid entry index");
                    entries.push(entry)
                }

                entries
            }
        }
    }


    //TODO Result = bool quorum-no-quorum
    fn send_append_entries(&self, entry : LogEntry) -> Result<(), Box<Error>>{
        if let NodeStatus::Leader = self.status {
            let cluster = self.cluster_configuration.lock()
                .expect("cluster lock is not poisoned");

            let (peers_list_copy, quorum_size) =
                (cluster.get_peers(self.id), cluster.get_quorum_size());

            let entry_index = entry.index;
            trace!("Node {} Sending 'Append Entries Request' Entry index={}", self.id, entry_index);
            let append_entries_request =  self.create_append_entry_request(AppendEntriesRequestType::NewEntry(entry));

            let replicate_log_to_peer_tx_clone = self.replicate_log_to_peer_tx.clone();
            let requester = |dest_node_id: u64, req: AppendEntriesRequest| {
                let resp_result = self.communicator.send_append_entries_request(dest_node_id, req);
                let resp = resp_result.expect("can get append_entries response"); //TODO check timeout

                trace!("Destination Node {} Append Entry (index={}) result={}",dest_node_id,  entry_index, resp.success);
                if !resp.success {
                    replicate_log_to_peer_tx_clone.send(dest_node_id).expect("can send replicate log msg");
                }

                Ok(resp)
            };

            let notify_peers_result = notify_peers(append_entries_request, self.id,peers_list_copy, Some(quorum_size), requester);
            return match notify_peers_result {
                Ok(_)=> Ok(()),
                Err(err) => Err(err)
            };
        }
        errors::new_err("send_append_entries failed: Not a leader".to_string(), None)
    }
}



use std::collections::HashMap;

use crossbeam_channel::{Sender};

use crate::communication::peers::{AppendEntriesRequest, PeerRequestHandler};
use crate::operation_log::{LogEntry,EntryContent};
use crate::common::peer_consensus_requester::request_peer_consensus;
use crate::rsm::{ReplicatedStateMachine};
use crate::operation_log::{OperationLog};
use crate::{errors, Cluster};
use crate::errors::{new_err, RaftError};


#[derive(Debug, Clone)]
//TODO decompose GOD object
//TODO decompose to Node & NodeState or extract get_peers() from cluster_config
pub struct Node<Log,Rsm,Pc, Ns,Cl>
where Log: OperationLog,
      Rsm: ReplicatedStateMachine,
      Pc : PeerRequestHandler,
      Ns : NodeStateSaver,
      Cl : Cluster{
    pub id : u64,
    current_term: u64,
    voted_for_id: Option<u64>,

    pub current_leader_id: Option<u64>,
    pub status : NodeStatus,
    next_index : HashMap<u64, u64>,
    commit_index: u64,

    pub log : Log,
    pub rsm : Rsm,
    communicator : Pc,
    state_saver: Ns,

    cluster_configuration : Cl,

    replicate_log_to_peer_tx: Sender<u64>,
    commit_index_updated_tx : Sender<u64>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NodeStatus {
    Follower,
    Candidate,
    Leader
}

#[derive(Copy, Clone, Debug)]
pub struct NodeState {
    pub node_id: u64,
    pub current_term: u64,
    pub vote_for_id: Option<u64>,
}

pub trait NodeStateSaver : Send + Sync + 'static{
    fn save_node_state(&self, state : NodeState) -> Result<(), RaftError>;
}

pub enum AppendEntriesRequestType {
    Heartbeat,
    NewEntry(LogEntry),
    UpdateNode(u64)
}

//TODO refactor to node_config
impl <Log, Rsm,Pc, Ns, Cl> Node<Log, Rsm,Pc, Ns, Cl>
where Log: OperationLog,
      Rsm: ReplicatedStateMachine,
      Pc : PeerRequestHandler,
      Ns : NodeStateSaver,
      Cl : Cluster{
    pub fn new(node_state: NodeState,
               log: Log,
               rsm: Rsm,
               communicator: Pc,
               cluster_configuration: Cl,
               state_saver : Ns,
               replicate_log_to_peer_tx: Sender<u64>,
               commit_index_updated_tx: Sender<u64>, ) -> Node<Log, Rsm, Pc, Ns, Cl> {
        Node {
            id: node_state.node_id,
            current_term: node_state.current_term,
            voted_for_id: node_state.vote_for_id,
            current_leader_id: None,
            status: NodeStatus::Follower,
            next_index: HashMap::new(),
            commit_index: 0,
            log,
            rsm,
            communicator,
            state_saver,
            cluster_configuration,
            replicate_log_to_peer_tx,
            commit_index_updated_tx,
        }
    }

    pub fn get_next_term(&self) -> u64 {self.current_term + 1}
    pub fn get_current_term(&self) -> u64 {
        if self.status == NodeStatus::Candidate {
            self.current_term + 1
        } else {
            self.current_term
        }
    }
    pub fn set_current_term(&mut self, new_term: u64) {
        self.current_term = new_term;
        self.save_node_state();
    }

    pub fn get_commit_index(&self) -> u64 {self.commit_index }
    pub fn set_commit_index(&mut self, new_commit_index: u64) {
        self.commit_index = new_commit_index;
        self.commit_index_updated_tx.send(new_commit_index)
            .expect("can send updated commit_index")
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
            let msg = format!("Node state save failed:{}", err);

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


    //TODO rename?
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

    //Check last log entry for voting purpose. Compares first term, index afterwards.
    //To grant vote - candidate log should contain entries same term or greater
    pub fn check_candidate_last_log_entry(&self,
                                          candidate_last_log_entry_term: u64,
                                          candidate_last_log_entry_index: u64) -> bool {
        if self.log.get_last_entry_term() > candidate_last_log_entry_term {
            return false
        }
        if self.log.get_last_entry_term() < candidate_last_log_entry_term {
            return true
        }
        //equal terms
        if self.log.get_last_entry_index() >  candidate_last_log_entry_index {
            return false
        }
        //last log entry of same or lesser term and same or lesser index
        true
    }

    pub fn append_entry_to_log(&mut self, entry: LogEntry) -> Result<(), RaftError> {
        let entry_index = entry.index;

        if self.log.get_last_entry_index() < entry_index {
            let log_append_result = self.log.append_entry(entry.clone());
            if let Err(err) = log_append_result {
                return errors::new_err(
                    format!("cannot append entry to log, index = {}", entry_index), err.to_string());
            }
        }
        Ok(())
    }


    pub fn append_content_to_log(&mut self, content: EntryContent) -> Result<bool, RaftError> {
        let entry = self.log.create_next_entry(self.get_current_term(), content);

        let send_result = self.send_append_entries(entry.clone());
        match send_result {
            Err(err) => {
                let msg = format!("Entry replication failed:{}", err);

                error!("{}", msg);
                return new_err("Cannot append content to log".to_string(), msg);
            },
            Ok(quorum_gathered) => {
                if !quorum_gathered {
                    warn!("send_append_entries unsuccessful: no quorum gathered");
                    return Ok(false)
                }
            }
        }

        let add_to_entry_result = self.append_entry_to_log(entry.clone());
        if let Err(err) = add_to_entry_result {
            let msg = format!("Append entry failed:{}", err);

            error!("{}", msg);
            return new_err("Cannot append content to log".to_string(), msg);
        }

        self.set_commit_index(entry.index);

        Ok(true)
    }


    fn send_append_entries(&self, entry : LogEntry) -> Result<bool, RaftError>{
        if let NodeStatus::Leader = self.status {

            let peers_copy = self.cluster_configuration.get_peers(self.id);
            let quorum_size = self.cluster_configuration.get_quorum_size();

            let entry_index = entry.index;
            trace!("Node {} Sending 'Append Entries Request' Entry index={}", self.id, entry_index);
            let append_entries_request =  self.create_append_entry_request(
                AppendEntriesRequestType::NewEntry(entry));

            let replicate_log_to_peer_tx_clone = self.replicate_log_to_peer_tx.clone();
            let requester = |dest_node_id: u64, req: AppendEntriesRequest| {
                let resp_result = self.communicator.send_append_entries_request(dest_node_id, req);
                match resp_result {
                    Ok(resp) => {
                        trace!("Destination Node {} Append Entry (index={}) result={}",
                               dest_node_id,  entry_index, resp.success);

                        //repeat on fail, if no consensus gathered - the replication won't happen
                        if !resp.success {
                            replicate_log_to_peer_tx_clone.send(dest_node_id).expect("can send replicate log msg");
                        }
                        Ok(resp)
                    },
                    Err(err) => {
                        trace!("Destination Node {} Append Entry (index={}) failed: {}",
                               dest_node_id,  entry_index, err);
                        Err(err)
                    }
                }
            };

            let notify_peers_result = request_peer_consensus(
                append_entries_request,
                self.id,
                peers_copy,
                Some(quorum_size), requester);

            return match notify_peers_result {
                Ok(quorum_gathered)=> Ok(quorum_gathered),
                Err(err) => Err(err)
            };
        }
        errors::new_err("send_append_entries failed: Not a leader".to_string(), String::new())
    }

    pub fn create_append_entry_request(&self, request_type : AppendEntriesRequestType)
        -> AppendEntriesRequest {
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

    fn get_prev_term_index(&self, entries: &[LogEntry]) -> (u64, u64) {
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
                let last_index = self.get_commit_index();

                let mut entries = Vec::new();
                for idx in  next_index..=last_index {
                    let entry = self.log.get_entry(idx).expect("valid entry index");
                    entries.push(entry)
                }

                entries
            }
        }
    }
}



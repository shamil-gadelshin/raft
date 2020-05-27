#[cfg(test)]
mod tests {

    use crate::node::state::Node;
    use crate::{
        AppendEntriesRequest, AppendEntriesResponse, Cluster, DataEntryContent, EntryContent,
        LogEntry, NodeState, NodeStateSaver, OperationLog, PeerRequestHandler, RaftError,
        ReplicatedStateMachine, VoteRequest, VoteResponse,
    };
    use crossbeam_channel::{Receiver, Sender};
    use std::sync::Arc;

    struct MockRsm;
    impl ReplicatedStateMachine for MockRsm {
        fn apply_entry(&mut self, _entry: LogEntry) -> Result<(), RaftError> {
            unimplemented!()
        }

        fn last_applied_entry_index(&self) -> u64 {
            unimplemented!()
        }
    }

    #[derive(Clone, Debug)]
    struct MockCluster;
    impl Cluster for MockCluster {
        fn quorum_size(&self) -> u32 {
            unimplemented!()
        }

        fn all_nodes(&self) -> Vec<u64> {
            unimplemented!()
        }

        fn peers(&self, _node_id: u64) -> Vec<u64> {
            unimplemented!()
        }
    }

    struct MockNodeStateServer;
    impl NodeStateSaver for MockNodeStateServer {
        fn save_node_state(&self, _state: NodeState) -> Result<(), RaftError> {
            unimplemented!()
        }
    }
    #[derive(Clone, Debug)]
    struct MockPeerRequestHandler;
    impl PeerRequestHandler for MockPeerRequestHandler {
        fn send_vote_request(
            &self,
            _destination_node_id: u64,
            _request: VoteRequest,
        ) -> Result<VoteResponse, RaftError> {
            unimplemented!()
        }

        fn send_append_entries_request(
            &self,
            _destination_node_id: u64,
            _request: AppendEntriesRequest,
        ) -> Result<AppendEntriesResponse, RaftError> {
            unimplemented!()
        }
    }

    #[derive(Clone, Debug)]
    pub struct MockOperationLog {
        last_index: u64,
        entries: Vec<LogEntry>,
    }

    impl MockOperationLog {
        pub fn new() -> MockOperationLog {
            MockOperationLog {
                entries: Vec::new(),
                last_index: 0,
            }
        }
    }

    impl OperationLog for MockOperationLog {
        fn create_next_entry(&mut self, term: u64, entry_content: EntryContent) -> LogEntry {
            LogEntry {
                index: self.last_index + 1,
                term,
                entry_content,
            }
        }

        fn append_entry(&mut self, entry: LogEntry) -> Result<(), RaftError> {
            if self.last_index < entry.index {
                self.entries.push(entry.clone());
                self.last_index = entry.index;
            }

            Ok(())
        }
        fn entry(&self, index: u64) -> Option<LogEntry> {
            let idx = index as usize;
            if idx > self.entries.len() || idx == 0 {
                return None;
            }
            Some(self.entries[idx - 1].clone())
        }

        fn last_entry_index(&self) -> u64 {
            let last = self.entries.last();
            if let Some(entry) = last {
                return entry.index;
            }

            0
        }
        fn last_entry_term(&self) -> u64 {
            let last = self.entries.last();
            if let Some(entry) = last {
                return entry.term;
            }

            0
        }
    }

    fn create_node_with_log(
        log: MockOperationLog,
    ) -> Node<MockOperationLog, MockRsm, MockPeerRequestHandler, MockNodeStateServer, MockCluster>
    {
        let node_state = NodeState {
            node_id: 1,
            current_term: 1,
            vote_for_id: Some(1),
        };

        let (mock_tx, _mock_rx): (Sender<u64>, Receiver<u64>) = crossbeam_channel::unbounded();
        Node::new(
            node_state,
            log,
            MockRsm {},
            MockPeerRequestHandler {},
            MockCluster {},
            MockNodeStateServer,
            mock_tx.clone(),
            mock_tx,
        )
    }

    #[test]
    fn test_empty_prev_term_index() {
        let log = MockOperationLog::new();

        let node = create_node_with_log(log);

        let (term, index) = node.prev_term_index(&Vec::new());

        assert_eq!(0, term);
        assert_eq!(0, index);
    }

    #[test]
    fn test_prev_term_index_with_single_record() {
        let mut log = MockOperationLog::new();
        log.append_entry(LogEntry {
            index: 1,
            term: 1,
            entry_content: EntryContent::Data(DataEntryContent {
                data: Arc::new(b"some"),
            }),
        }).unwrap();

        let node = create_node_with_log(log);

        let (term, index) = node.prev_term_index(&Vec::new());

        assert_eq!(0, term);
        assert_eq!(0, index);
    }

    #[test]
    fn test_prev_term_index_for_heartbeat() {
        let mut log = MockOperationLog::new();
        log.append_entry(LogEntry {
            index: 1,
            term: 1,
            entry_content: EntryContent::Data(DataEntryContent {
                data: Arc::new(b"some"),
            }),
        }).unwrap();

        log.append_entry(LogEntry {
            index: 2,
            term: 2,
            entry_content: EntryContent::Data(DataEntryContent {
                data: Arc::new(b"some"),
            }),
        }).unwrap();

        let node = create_node_with_log(log);

        let (term, index) = node.prev_term_index(&Vec::new());

        assert_eq!(2, term);
        assert_eq!(2, index);
    }

    #[test]
    fn test_prev_term_index_for_entry() {
        let mut log = MockOperationLog::new();
        log.append_entry(LogEntry {
            index: 1,
            term: 1,
            entry_content: EntryContent::Data(DataEntryContent {
                data: Arc::new(b"some"),
            }),
        }).unwrap();

        log.append_entry(LogEntry {
            index: 2,
            term: 2,
            entry_content: EntryContent::Data(DataEntryContent {
                data: Arc::new(b"some"),
            }),
        }).unwrap();

        let entry = log.entry(log.last_entry_index()).unwrap();
        let node = create_node_with_log(log);

        let (term, index) = node.prev_term_index(&vec![entry]);

        assert_eq!(1, term);
        assert_eq!(1, index);
    }
}

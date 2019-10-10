use raft::{NodeState, NodeStateSaver, RaftError};

/// Mock for the NodeStateSaver state. Logs only - no persistence.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Default)]
pub struct MockNodeStateSaver;

impl NodeStateSaver for MockNodeStateSaver {
    fn save_node_state(&self, state: NodeState) -> Result<(), RaftError> {
        info!("Node state saved: {}", state);

        Ok(())
    }
}

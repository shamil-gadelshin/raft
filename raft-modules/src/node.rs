use raft::{NodeStateSaver, NodeState, RaftError};

#[derive(Default)]
pub struct MockNodeStateSaver;

impl NodeStateSaver for MockNodeStateSaver {
	fn save_node_state(&self, state: NodeState) -> Result<(), RaftError> {
		info!("Node state saved: {:?}", state);

		Ok(())
	}
}
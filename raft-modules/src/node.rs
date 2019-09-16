use raft::{NodeStateSaver, NodeState};
use std::error::Error;

#[derive(Default)]
pub struct MockNodeStateSaver;

impl NodeStateSaver for MockNodeStateSaver {
	fn save_node_state(&self, state: NodeState) -> Result<(), Box<Error>> {
		println!("Node state saved: {:?}", state);

		Ok(())
	}
}
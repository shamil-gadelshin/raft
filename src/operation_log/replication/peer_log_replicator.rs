use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, Sender};

use crate::operation_log::LogStorage;
use crate::state::{Node, NodeStatus, AppendEntriesRequestType};
use crate::communication::peers::InProcPeerCommunicator;
use crate::fsm::Fsm;


pub struct LogReplicatorParams<Log, FsmT>
	where Log: Sync + Send + LogStorage + 'static, FsmT: Sync + Send + Fsm + 'static {
	pub protected_node : Arc<Mutex<Node<Log, FsmT>>>,
	pub replicate_log_to_peer_rx: Receiver<u64>,
	pub replicate_log_to_peer_tx: Sender<u64>,
	pub communicator : InProcPeerCommunicator
}


pub fn replicate_log_to_peer<Log, FsmT>(params : LogReplicatorParams<Log, FsmT>)
	where Log: Sync + Send + LogStorage + 'static, FsmT: Sync + Send + Fsm + 'static {
	loop {
		let peer_id = params.replicate_log_to_peer_rx.recv()
			.expect("can receive peer_id from replicate_log_to_peer_rx");

		trace!("Replicate request for peer {}", peer_id);

		let mut node = params.protected_node.lock().expect("node lock is not poisoned");

		if node.status != NodeStatus::Leader {
			warn!("Obsolete (Not a Leader) replicate log request - Node ({:?}) to peer ({:?}) ", node.id, peer_id );
			continue;
		}

		let next_index = node.get_next_index(peer_id);

		let append_entries_request = node.create_append_entry_request(AppendEntriesRequestType::UpdateNode(peer_id));
		let request_entry_count  = append_entries_request.entries.len() as u64;

		if request_entry_count > 0 {
			let resp_result = params.communicator.send_append_entries_request(peer_id, append_entries_request);

			//TODO check result
			let resp = resp_result.expect("can get append_entries response");

			if !resp.success {
				if next_index > 1 {
					let modified_next_index = next_index - 1;
					node.set_next_index(peer_id, modified_next_index);
				} else {
					//TODO delete warning?
					warn!("Unsuccessful replicate log request and next_index <= 1  - Node ({:?}) to peer ({:?}) ", node.id, peer_id);
				}

				params.replicate_log_to_peer_tx.send(peer_id).expect("can send update peer log request")
			} else {
				node.set_next_index(peer_id, next_index + request_entry_count)
			}
		} else {
			trace!("Replication: no new entries for Node {}", peer_id)
		}
	}
}
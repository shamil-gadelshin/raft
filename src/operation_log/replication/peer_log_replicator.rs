use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, Sender};

use crate::operation_log::storage::LogStorage;
use crate::state::{Node, NodeStatus, AppendEntriesRequestType};
use crate::communication::peers::InProcNodeCommunicator;
use crate::leadership::leader_watcher::watch_leader_status;


pub fn replicate_log_to_peer<Log>(protected_node: Arc<Mutex<Node<Log>>>,
								  replicate_log_to_peer_rx: Receiver<u64>,
								  replicate_log_to_peer_tx: Sender<u64>,
								  communicator : InProcNodeCommunicator)
where Log: Sync + Send + LogStorage {
	loop {
		let peer_id = replicate_log_to_peer_rx.recv()
			.expect("can receive peer_id from replicate_log_to_peer_rx");

		let mut node = protected_node.lock().expect("node lock is not poisoned");

		if node.status != NodeStatus::Leader {
			warn!("Obsolete (Not a Leader) replicate log request - Node ({:?}) to peer ({:?}) ", node.id, peer_id );
			continue;
		}

		let next_index = {
			if !node.next_index.contains_key(&peer_id){
				0
			} else {
				node.next_index[&peer_id]
			}
		};

		let append_entries_request = node.create_append_entry_request(AppendEntriesRequestType::UpdateNode(peer_id));
		let resp_result = communicator.send_append_entries_request(peer_id, append_entries_request);

		//TODO check result
		let resp = resp_result.expect("can get append_entries response");

		if !resp.success {
			if next_index > 1{
				let modified_next_index = next_index - 1;

				(*(node.next_index.entry(peer_id).or_insert(modified_next_index))) = modified_next_index;
			} else {
				//TODO delete warning?
				warn!("Unsuccessful replicate log request and next_index <= 1  - Node ({:?}) to peer ({:?}) ", node.id, peer_id );
			}

			replicate_log_to_peer_tx.send(peer_id).expect("can send update peer log request")
		}
	}
}
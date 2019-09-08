use std::sync::{Arc, Mutex};

use crossbeam_channel::Receiver;

use crate::operation_log::LogStorage;
use crate::fsm::{Fsm};
use crate::common::LogEntry;
use crate::state::Node;

pub struct FsmUpdaterParams<Log, FsmT>
	where Log: Sync + Send + LogStorage + 'static, FsmT: Sync + Send + Fsm + 'static {
	pub protected_node : Arc<Mutex<Node<Log,FsmT>>>,
	pub commit_index_updated_rx : Receiver<u64>
}


pub fn update_fsm<Log, FsmT:  Sync + Send + Fsm>(params : FsmUpdaterParams<Log,FsmT>)
	where Log: Sync + Send + LogStorage + 'static, FsmT: Sync + Send + Fsm + 'static{
	loop {
		let new_commit_index = params.commit_index_updated_rx.recv().expect("valid receive of commit index"); //fsm should be updated
		let mut node = params.protected_node.lock().expect("node lock is not poisoned");
		trace!("Update Fsm request for Node {}, Commit index ={}", node.id, new_commit_index);
		loop {
			let last_applied = node.fsm.get_last_applied_entry_index();

			let mut entry_index= 0;
			let entry_result : Option<LogEntry>  = {
				if new_commit_index > last_applied {
					entry_index = last_applied + 1;

					node.log.get_entry(entry_index)
				} else {
					None
				}
			};

			if let Some(entry) = entry_result {
				let fsm_apply_result = node.fsm.apply_entry(entry);

				if let Err(err) = fsm_apply_result {
					error!("Fsm Apply error. Entry = {}: {}", entry_index, err.description());
					break;
            	}
			} else {
				break;
			}
		}
	}

}
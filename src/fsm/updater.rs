use std::sync::{Arc, Mutex};

use crossbeam_channel::Receiver;

use crate::operation_log::LogStorage;
use crate::fsm::{Fsm};
use crate::common::LogEntry;

pub fn update_fsm<Log, FsmT:  Sync + Send + Fsm>(protected_log : Arc<Mutex<Log>>, protected_fsm : Arc<Mutex<FsmT>>, update_fsm_rx: Receiver<bool>)
	where Log: Sync + Send + LogStorage + 'static{
	loop {
		update_fsm_rx.recv().expect("valid receive"); //fsm should be updated

		loop {
			let mut fsm = protected_fsm.lock().expect("fsm lock is not poisoned");
			let last_applied = fsm.get_last_applied_entry_index();

			let entry_result : Option<LogEntry>  = {
				let log = protected_log.lock().expect("fsm lock is not poisoned");

				if log.get_last_entry_index() > last_applied {
					let index = last_applied + 1;

					log.get_entry(index)
				} else {
					None
				}
			};

			if let Some(entry) = entry_result {
				fsm.apply_entry(entry);
			} else {
				break;
			}
		}
	}

}
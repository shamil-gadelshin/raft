pub mod updater;

use std::sync::{Mutex, Arc};
use std::error::Error;

use crate::configuration::cluster::ClusterConfiguration;
use crate::common::{LogEntry};

pub trait Fsm {
	fn apply_entry(&mut self, entry: LogEntry) -> Result<(), Box<Error>>;
	fn get_last_applied_entry_index(&self) -> u64;
}

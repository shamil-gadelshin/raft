#[macro_use] extern crate log;
#[macro_use] extern crate crossbeam_channel;

mod common;
mod leadership;
mod communication;
mod operation_log;
mod configuration;
mod state;
mod fsm;
mod request_handler;
mod workers;
mod errors;

pub use communication::client::{AddServerRequest,NewDataRequest, InProcClientCommunicator};
pub use communication::peers::{InProcNodeCommunicator};
pub use operation_log::storage::{MemoryLogStorage, LogStorage};
pub use configuration::cluster::ClusterConfiguration;
pub use configuration::node::NodeConfiguration;

use std::thread::JoinHandle;

pub fn start_node<Log>(node_config : NodeConfiguration, log_storage : Log ) -> JoinHandle<()>
where Log: Sync + Send + LogStorage + 'static{
	workers::node_main_process::run_thread(node_config, log_storage)
}


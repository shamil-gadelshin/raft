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
mod node;

pub use communication::client::{AddServerRequest,NewDataRequest, InProcClientCommunicator};
pub use communication::peers::InProcPeerCommunicator;
pub use operation_log::{LogStorage};
pub use configuration::cluster::ClusterConfiguration;
pub use configuration::node::NodeConfiguration;
pub use fsm::Fsm;
pub use common::{LogEntry, DataEntryContent, EntryContent};

use std::thread::JoinHandle;

pub fn start_node<Log, FsmT>(node_config : NodeConfiguration, log_storage : Log, fsm : FsmT ) -> JoinHandle<()>
where Log: Sync + Send + LogStorage + 'static, FsmT:  Sync + Send + Fsm+ 'static{
	node::start(node_config, log_storage, fsm)
}


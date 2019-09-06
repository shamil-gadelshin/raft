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
pub mod workers;
mod errors;

pub use communication::client::{AddServerRequest,NewDataRequest, InProcClientCommunicator};
pub use communication::peers::{InProcNodeCommunicator};
pub use operation_log::storage::{MemoryLogStorage};
pub use configuration::cluster::ClusterConfiguration;
pub use configuration::node::NodeConfiguration;
pub use workers::node_main_process;


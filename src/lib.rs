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
mod errors;
mod node;


pub use communication::client::{AddServerRequest,NewDataRequest, ClientRpcResponse, ClientResponseStatus,ClientRequestHandler, ClientRequestChannels};
pub use operation_log::{LogStorage};
pub use configuration::cluster::ClusterConfiguration;
pub use configuration::node::NodeConfiguration;
pub use fsm::Fsm;
pub use common::{LogEntry, DataEntryContent, EntryContent};
pub use crate::communication::peers::{PeerRequestHandler, PeerRequestChannels};
pub use crate::communication::peers::{InProcPeerCommunicator};


use std::thread::JoinHandle;

pub fn start_node<Log, FsmT,Cc, Pc >(node_config : NodeConfiguration<Cc,Pc>, log_storage : Log, fsm : FsmT ) -> JoinHandle<()>
where Log: Sync + Send + LogStorage + 'static,
	  FsmT:  Sync + Send + Fsm+ 'static,
	  Cc : Sync + Send + 'static + Clone  +  ClientRequestChannels,
	  Pc : Sync + Send + 'static + PeerRequestHandler + PeerRequestChannels + Clone{
	node::start(node_config, log_storage, fsm)
}


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
pub use operation_log::{OperationLog};
pub use configuration::cluster::ClusterConfiguration;
pub use configuration::node::NodeConfiguration;
pub use fsm::FiniteStateMachine;
pub use common::{LogEntry, DataEntryContent, EntryContent};
pub use communication::peers::{VoteRequest, VoteResponse, AppendEntriesRequest, AppendEntriesResponse};
pub use communication::peers::{PeerRequestHandler, PeerRequestChannels};
pub use state::NodeState;
pub use configuration::node::ElectionTimer;
pub use state::NodeStateSaver;
pub use common::Worker;

use crossbeam_channel::{Receiver, Sender};



pub fn start_node<Log, Fsm,Cc, Pc, Et, Ns >(node_config : NodeConfiguration<Cc,Pc, Et>,
											log_storage : Log,
											fsm : Fsm,
											state_saver : Ns
) -> Worker
where Log: OperationLog ,
	  Fsm: FiniteStateMachine,
	  Cc : ClientRequestChannels,
	  Pc : PeerRequestHandler + PeerRequestChannels,
	  Et : ElectionTimer,
	  Ns : NodeStateSaver{

	let (terminate_worker_tx, terminate_worker_rx): (Sender<()>, Receiver<()>) = crossbeam_channel::unbounded();
	let join_handle = node::start(node_config, log_storage, fsm, state_saver, terminate_worker_rx);

	Worker{join_handle, terminate_worker_tx}
}


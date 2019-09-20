#[macro_use] extern crate log;
#[macro_use] extern crate crossbeam_channel;
extern crate rayon;

mod common;
mod leadership;
mod communication;
mod operation_log;
mod configuration;
mod state;
mod rsm;
mod request_handler;
mod node;
mod errors;


pub use communication::client::{AddServerRequest,NewDataRequest, ClientRpcResponse, ClientResponseStatus,ClientRequestHandler, ClientRequestChannels};
pub use operation_log::{OperationLog};
pub use configuration::node::Cluster;
pub use configuration::node::NodeConfiguration;
pub use rsm::ReplicatedStateMachine;
pub use common::{LogEntry, DataEntryContent, EntryContent};
pub use communication::peers::{VoteRequest, VoteResponse, AppendEntriesRequest, AppendEntriesResponse};
pub use communication::peers::{PeerRequestHandler, PeerRequestChannels};
pub use state::NodeState;
pub use configuration::node::ElectionTimer;
pub use state::NodeStateSaver;
pub use configuration::node::NodeTimings;
pub use errors::{RaftError, new_err};
pub type NodeWorker = common::RaftWorker;


pub fn start_node<Log, Rsm,Cc, Pc, Et, Ns, Cl>(node_config : NodeConfiguration<Log, Rsm,Cc, Pc, Et, Ns, Cl>) -> NodeWorker
where Log: OperationLog ,
	  Rsm: ReplicatedStateMachine,
	  Cc : ClientRequestChannels,
	  Pc : PeerRequestHandler + PeerRequestChannels,
	  Et : ElectionTimer,
	  Ns : NodeStateSaver,
	  Cl : Cluster{

	common::run_worker(node::start, node_config)
}


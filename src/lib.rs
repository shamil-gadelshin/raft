//! This crate is Raft consensus implementation in Rust.
//! It is based on [Diego Ongaro's dissertation](https://github.com/shamil-gadelshin/raft/blob/master/doc/raft_dissertation.pdf).

#![warn(missing_debug_implementations, unsafe_code)]
#![forbid(missing_docs)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate crossbeam_channel;
#[macro_use]
extern crate derive_more;

mod common;
mod communication;
mod errors;
mod leadership;
mod node;
mod operation_log;
mod request_handler;
mod rsm;

pub use communication::client::{
    AddServerRequest, ClientRequestChannels, ClientRequestHandler, ClientResponseStatus,
    ClientRpcResponse, NewDataRequest,
};
pub use communication::peers::{
    AppendEntriesRequest, AppendEntriesResponse, PeerRequestChannels, PeerRequestHandler,
    VoteRequest, VoteResponse,
};
pub use errors::{new_err, RaftError};
pub use leadership::ElectionTimer;
pub use node::configuration::{Cluster, NodeConfiguration, NodeLimits};
pub use node::state::{NodeState, NodeStateSaver};
pub use operation_log::{
    DataEntryContent, EntryContent, LogEntry, NewClusterConfigurationEntryContent, OperationLog,
};
pub use rsm::ReplicatedStateMachine;

/// Type alias for RaftWorker.
pub type NodeWorker = common::RaftWorker;

/// Starts node with provided configuration. Node starts in separate thread.
/// Returns [NodeWorker](type.NodeWorker.html)
pub fn start_node<Log, Rsm, Cc, Pc, Et, Ns, Cl>(
    node_config: NodeConfiguration<Log, Rsm, Cc, Pc, Et, Ns, Cl>,
) -> NodeWorker
where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Cc: ClientRequestChannels,
    Pc: PeerRequestHandler + PeerRequestChannels,
    Et: ElectionTimer,
    Ns: NodeStateSaver,
    Cl: Cluster,
{
    common::run_worker(node::start, node_config)
}

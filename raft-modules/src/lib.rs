//! # Raft Modules
//!
//! This subproject of the [raft](https://github.com/shamil-gadelshin/raft) provides examples or
//! test implementations of the Raft Modules.
//! 
//! ## Modules
//! 
//! - **Operation log** - supports operation log entries processing and calculating parameters (index, term, etc).
//! - **Replicated state machine** - supports operations with replicated state machine (like 'append new entry').
//! - **Peer request handler** - module responsible for communication between Raft Nodes.
//! - **Client request handler** - this module responsible for client communication. Module handles request like 'add new data' or 'add server'.
//! - **Cluster** - defines cluster membership and quorum rules.
//! - **Election timer** - calculates duration to the next elections.
//! - **Node state saver** - responsible for node state persistence.
//! 
//! 
//! ## Implementations
//! 
//! - Operation log - **MemoryOperationLog** - memory based operation log. No persistence. For testing.
//! - Replicated state machine - **MemoryRsm** - memory based state machine emulation. No persistence. For testing.
//! - Peer request handler 
//!   + **InProcPeerCommunicator** - channel-based implementation. For in-process testing.
//!   + **NetworkPeerCommunicator** - grpc-based implementation. Can be used as template for real-case.
//! - Client request handler
//!   + **InProcClientCommunicator** - channel-based implementation. For in-process testing.
//!   + **NetworkClientCommunicator** - grpc-based implementation. Can be used as template for real-case.
//! - Cluster - **ClusterConfiguration** - calculates quorum as majority.
//! - Election timer
//!   + **RandomizedElectionTimer** - provides random time duration within a range.
//!   + **FixedElectionTimer** - provides fixed time duration. For 'guaranteed leadership' tests.
//! - Node state saver - **MockNodeStateSaver** - mock implentation of node state persistence.
//!   

#![forbid(missing_docs)]

#[macro_use]
extern crate log;
extern crate crossbeam_channel;
extern crate raft;

mod cluster;
mod communication;
mod election;
mod memory_log;
mod memory_rsm;
mod node;

pub use cluster::ClusterConfiguration;
pub use communication::duplex_channel::DuplexChannel;
pub use communication::inproc::inproc_client_communicator::InProcClientCommunicator;
pub use communication::inproc::inproc_peer_communicator::InProcPeerCommunicator;
pub use communication::network::client_communicator::network_client_communicator::NetworkClientCommunicator;
pub use communication::network::peer_communicator::network_peer_communicator::NetworkPeerCommunicator;
pub use communication::network::peer_communicator::service_discovery::static_table::StaticTableServiceDiscovery;
pub use election::fixed_election_timer::FixedElectionTimer;
pub use election::randomized_election_timer::RandomizedElectionTimer;
pub use memory_log::MemoryOperationLog;
pub use memory_rsm::MemoryRsm;
pub use node::MockNodeStateSaver;

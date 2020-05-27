use std::time::Duration;

use crate::communication::client::ClientRequestChannels;
use crate::communication::peers::PeerRequestChannels;
use crate::node::state::NodeState;
use crate::{ElectionTimer, NodeStateSaver, PeerRequestHandler};
use crate::{OperationLog, ReplicatedStateMachine};

/// Constraints for the Raft node.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Display)]
#[display(
    fmt = "Node Limits: heartbeat_timeout {:?} communication_timeout {:?} max_data_size {}",
    heartbeat_timeout,
    communication_timeout,
    max_data_content_size
)]
pub struct NodeLimits {
    /// The follower node starts new elections and converts to candidate when this timeout expires.
    pub heartbeat_timeout: Duration,

    /// Peer and clients communication timeout during the recv & send operations with channels.
    pub communication_timeout: Duration,

    /// Max data content size that can be accepted with client AddData request.
    pub max_data_content_size: u64,
}

impl Default for NodeLimits {
    fn default() -> Self {
        NodeLimits {
            heartbeat_timeout: Duration::from_millis(800),
            communication_timeout: Duration::from_millis(1000),
            max_data_content_size: 20 * 1024 * 1024, //20 MB
        }
    }
}

/// Cluster configuration administrator.
pub trait Cluster: Send + Sync + Clone + 'static {
    /// Returns the quorum size of the current cluster. Can be used during elections or
    /// replication requests.
    fn quorum_size(&self) -> u32;

    /// Returns all current nodes of the cluster.
    fn all_nodes(&self) -> Vec<u64>;

    /// Returns peers for the node.
    fn peers(&self, node_id: u64) -> Vec<u64>;
}

/// Node starting configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Display)]
#[display(fmt = "Node configuration: node_id {}", "node_state.node_id")]
pub struct NodeConfiguration<Log, Rsm, Cc, Pc, Et, Ns, Cl>
where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Cc: ClientRequestChannels,
    Pc: PeerRequestHandler + PeerRequestChannels,
    Et: ElectionTimer,
    Ns: NodeStateSaver,
    Cl: Cluster,
{
    /// The persistence state for the node.
    pub node_state: NodeState,

    /// Known cluster configuration.
    pub cluster_configuration: Cl,

    /// Peer communication provider.
    pub peer_communicator: Pc,

    /// Client communication provider.
    pub client_communicator: Cc,

    /// Election timeout timer.
    pub election_timer: Et,

    /// Operation log administrator.
    pub operation_log: Log,

    /// Replicated state machine administrator.
    pub rsm: Rsm,

    /// Responsible for the persistent of the node state.
    pub state_saver: Ns,

    /// Node constraints. Timeouts and data sizes.
    pub limits: NodeLimits,
}

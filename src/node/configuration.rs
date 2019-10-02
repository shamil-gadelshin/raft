use std::time::Duration;

use crate::communication::peers::{PeerRequestChannels};
use crate::communication::client::{ClientRequestChannels};
use crate::node::state::NodeState;
use crate::{OperationLog, ReplicatedStateMachine, PeerRequestHandler, NodeStateSaver, ElectionTimer};


#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct NodeLimits {
    pub heartbeat_timeout: Duration,
    pub communication_timeout: Duration,
    pub max_data_content_size: u64
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

pub trait Cluster : Send + Sync + Clone + 'static{
    fn get_quorum_size(&self) -> u32;
    fn get_all_nodes(&self) -> Vec<u64>;
    fn get_peers(&self, node_id : u64) -> Vec<u64>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct NodeConfiguration<Log, Rsm, Cc, Pc, Et, Ns, Cl>
    where Log: OperationLog ,
          Rsm: ReplicatedStateMachine,
          Cc : ClientRequestChannels,
          Pc : PeerRequestHandler + PeerRequestChannels,
          Et : ElectionTimer,
          Ns : NodeStateSaver,
          Cl : Cluster{
    pub node_state: NodeState,
    pub cluster_configuration: Cl,
    pub peer_communicator: Pc,
    pub client_communicator: Cc,
    pub election_timer: Et,
    pub operation_log : Log,
    pub rsm : Rsm,
    pub state_saver : Ns,
    pub limits: NodeLimits
}
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::configuration::cluster::{ClusterConfiguration};
use crate::communication::peers::{PeerRequestChannels};
use crate::communication::client::{ClientRequestChannels};
use crate::state::NodeState;
use crate::{OperationLog, FiniteStateMachine, PeerRequestHandler, NodeStateSaver};


pub trait ElectionTimer: Send + 'static {
    fn get_next_elections_timeout(&self) -> Duration;
}

pub struct NodeTimings {
    pub heartbeat_timeout: Duration,
    pub communication_timeout: Duration,
}

impl Default for NodeTimings{
    fn default() -> Self {
        NodeTimings{
            heartbeat_timeout: Duration::from_millis(1000),
            communication_timeout: Duration::from_millis(1000),
        }
    }
}

pub struct NodeConfiguration<Log, Fsm, Cc, Pc, Et, Ns>
    where Log: OperationLog ,
          Fsm: FiniteStateMachine,
          Cc : ClientRequestChannels,
          Pc : PeerRequestHandler + PeerRequestChannels,
          Et : ElectionTimer,
          Ns : NodeStateSaver {
    pub node_state: NodeState,
    pub cluster_configuration: Arc<Mutex<ClusterConfiguration>>,
    pub peer_communicator: Pc,
    pub client_communicator: Cc,
    pub election_timer: Et,
    pub operation_log : Log,
    pub fsm : Fsm,
    pub state_saver : Ns,
    pub timings: NodeTimings
}
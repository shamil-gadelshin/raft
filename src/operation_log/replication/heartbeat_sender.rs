use std::sync::{Arc, Mutex};
use std::time::Duration;
use crossbeam_channel::{Receiver};


use crate::state::{Node, NodeStatus, AppendEntriesRequestType, NodeStateSaver};
use crate::communication::peers::{AppendEntriesRequest, PeerRequestHandler};
use crate::common::peer_notifier::notify_peers;
use crate::configuration::cluster::{ClusterConfiguration};
use crate::operation_log::OperationLog;
use crate::fsm::FiniteStateMachine;

pub struct SendHeartbeatAppendEntriesParams<Log, Fsm, Pc,Ns>
    where Log: OperationLog,
          Fsm: FiniteStateMachine,
          Pc : PeerRequestHandler,
          Ns : NodeStateSaver{
    pub protected_node: Arc<Mutex<Node<Log, Fsm, Pc, Ns>>>,
    pub cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
    pub communicator : Pc,
    pub leader_initial_heartbeat_rx : Receiver<bool>,
}

//TODO remove clone-values
//TODO park-unpark the thread
pub fn send_heartbeat_append_entries<Log, Fsm, Pc, Ns>(params : SendHeartbeatAppendEntriesParams<Log, Fsm, Pc, Ns>)
    where Log: OperationLog,
          Fsm: FiniteStateMachine,
          Pc : PeerRequestHandler,
          Ns : NodeStateSaver{
    loop {
        let heartbeat_timeout = crossbeam_channel::after(leader_heartbeat_duration_ms());
        select!(
            recv(heartbeat_timeout) -> _  => {
                send_heartbeat(params.protected_node.clone(), params.cluster_configuration.clone(), &params.communicator)
                },
            recv(params.leader_initial_heartbeat_rx) -> _  => {
                trace!("Sending initial heartbeat...");
                send_heartbeat(params.protected_node.clone(), params.cluster_configuration.clone(), &params.communicator)
                },
        );
    }
}

fn send_heartbeat<Log, Fsm, Pc, Ns>(protected_node : Arc<Mutex<Node<Log, Fsm, Pc, Ns>>>,
                             cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
                             communicator : &Pc)
    where Log: OperationLog,
          Fsm: FiniteStateMachine,
          Pc : PeerRequestHandler,
          Ns : NodeStateSaver {
    let node = protected_node.lock().expect("node lock is not poisoned");

    if let NodeStatus::Leader = node.status {
        let cluster = cluster_configuration.lock().expect("cluster lock is not poisoned");

        let peers_list_copy = cluster.get_peers(node.id);

        let append_entries_heartbeat =
            node.create_append_entry_request(AppendEntriesRequestType::Heartbeat);

        trace!("Node {:?} Send 'empty Append Entries Request(heartbeat)'.", node.id);

        let requester = |dest_node_id: u64, req: AppendEntriesRequest| communicator.send_append_entries_request(dest_node_id, req);
        let result = notify_peers(append_entries_heartbeat, node.id, peers_list_copy, None, requester);

        if result.is_err() {
            error!("Node {:?} Send heartbeat failed: {}", node.id, result.unwrap_err().description())
        }
    }
}

//TODO move to node_config
fn leader_heartbeat_duration_ms() -> Duration{
    Duration::from_millis(1000)
}

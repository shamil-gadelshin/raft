use std::sync::{Arc, Mutex};
use std::time::Duration;
use crossbeam_channel::{Receiver};


use crate::state::{Node, NodeStatus, AppendEntriesRequestType, NodeStateSaver};
use crate::communication::peers::{AppendEntriesRequest, PeerRequestHandler};
use crate::common::peer_consensus_requester::request_peer_consensus;
use crate::configuration::cluster::{ClusterConfiguration};
use crate::operation_log::OperationLog;
use crate::rsm::ReplicatedStateMachine;

pub struct SendHeartbeatAppendEntriesParams<Log, Rsm, Pc,Ns>
    where Log: OperationLog,
          Rsm: ReplicatedStateMachine,
          Pc : PeerRequestHandler,
          Ns : NodeStateSaver{
    pub protected_node: Arc<Mutex<Node<Log, Rsm, Pc, Ns>>>,
    pub cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
    pub communicator : Pc,
    pub leader_initial_heartbeat_rx : Receiver<bool>,
    pub heartbeat_timeout : Duration
}

//TODO park-unpark the thread
pub fn send_heartbeat_append_entries<Log, Rsm, Pc, Ns>(params : SendHeartbeatAppendEntriesParams<Log, Rsm, Pc, Ns>,
                                                       terminate_worker_rx : Receiver<()>)
    where Log: OperationLog,
          Rsm: ReplicatedStateMachine,
          Pc : PeerRequestHandler,
          Ns : NodeStateSaver{
    info!("Heartbeat sender worker started");
    loop {
        let heartbeat_timeout = crossbeam_channel::after(params.heartbeat_timeout);
        select!(
            recv(terminate_worker_rx) -> res  => {
                if res.is_err() {
                    error!("Abnormal exit for heartbeat sender worker");
                }
                break
            },
            recv(heartbeat_timeout) -> _  => {
                send_heartbeat(params.protected_node.clone(), params.cluster_configuration.clone(), &params.communicator)
            },
            recv(params.leader_initial_heartbeat_rx) -> leader_initial_heartbeat_result  => {
                if let Err(err) = leader_initial_heartbeat_result {
                    error!("Invalid result from leader_initial_heartbeat_rx: {}", err);
                }
                trace!("Sending initial heartbeat...");
                send_heartbeat(params.protected_node.clone(), params.cluster_configuration.clone(), &params.communicator)
             },
        );
    }
    info!("Heartbeat sender worker stopped");
}

fn send_heartbeat<Log, Rsm, Pc, Ns>(protected_node : Arc<Mutex<Node<Log, Rsm, Pc, Ns>>>,
                             cluster_configuration : Arc<Mutex<ClusterConfiguration>>,
                             communicator : &Pc)
    where Log: OperationLog,
          Rsm: ReplicatedStateMachine,
          Pc : PeerRequestHandler,
          Ns : NodeStateSaver {
    let node = protected_node.lock().expect("node lock is not poisoned");

    if let NodeStatus::Leader = node.status {
        let cluster = cluster_configuration.lock().expect("cluster lock is not poisoned");

        let peers_list_copy = cluster.get_peers(node.id);

        let append_entries_heartbeat =
            node.create_append_entry_request(AppendEntriesRequestType::Heartbeat);

        trace!("Node {} Send 'empty Append Entries Request(heartbeat)'", node.id);

        let requester = |dest_node_id: u64, req: AppendEntriesRequest| communicator.send_append_entries_request(dest_node_id, req);
        let result = request_peer_consensus(append_entries_heartbeat, node.id, peers_list_copy, None, requester);

        if result.is_err() {
            error!("Node {} Send heartbeat failed: {}", node.id, result.unwrap_err())
        }
    }
}

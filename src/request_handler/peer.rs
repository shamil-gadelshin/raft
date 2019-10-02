use crossbeam_channel::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::communication::peers::{AppendEntriesRequest, VoteRequest};
use crate::communication::peers::{PeerRequestChannels, PeerRequestHandler};
use crate::leadership::node_leadership_fsm::LeaderElectionEvent;
use crate::leadership::vote_request_processor::process_vote_request;
use crate::leadership::LeaderConfirmationEvent;
use crate::node::state::{Node, NodeStateSaver};
use crate::operation_log::replication::append_entries_processor::process_append_entries_request;
use crate::operation_log::OperationLog;
use crate::rsm::ReplicatedStateMachine;
use crate::Cluster;

pub struct PeerRequestHandlerParams<Log, Rsm, Pc, Ns, Cl>
where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Pc: PeerRequestChannels + PeerRequestHandler,
    Ns: NodeStateSaver,
    Cl: Cluster,
{
    pub protected_node: Arc<Mutex<Node<Log, Rsm, Pc, Ns, Cl>>>,
    pub peer_communicator: Pc,
    pub leader_election_event_tx: Sender<LeaderElectionEvent>,
    pub reset_leadership_watchdog_tx: Sender<LeaderConfirmationEvent>,
    pub communication_timeout: Duration,
}

pub fn process_peer_request<Log, Rsm, Pc, Ns, Cl>(
    params: PeerRequestHandlerParams<Log, Rsm, Pc, Ns, Cl>,
    terminate_worker_rx: Receiver<()>,
) where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Pc: PeerRequestChannels + PeerRequestHandler,
    Ns: NodeStateSaver,
    Cl: Cluster,
{
    info!("Peer request processor worker started");
    let node_id = {
        params
            .protected_node
            .lock()
            .expect("node lock is not poisoned")
            .id
    };
    let vote_request_rx = params.peer_communicator.vote_request_rx(node_id).clone();
    let append_entries_request_rx = params.peer_communicator.append_entries_request_rx(node_id);

    loop {
        select!(
            recv(terminate_worker_rx) -> res  => {
                if res.is_err() {
                    error!("Abnormal exit for client request processor worker");
                }
                break
            },
            recv(vote_request_rx) -> res => {
                let request = res.expect("can get request from vote_request_rx");

                handle_vote_request(node_id,request,  &params);
            },
            recv(append_entries_request_rx) -> res => {
                let request = res.expect("can get request from append_entries_request_rx");

                handle_append_entries_request(node_id, request,  &params);
            }
        );
    }
    info!("Peer request processor worker stopped");
}

fn handle_vote_request<Log, Rsm, Pc, Ns, Cl>(
    node_id: u64,
    request: VoteRequest,
    params: &PeerRequestHandlerParams<Log, Rsm, Pc, Ns, Cl>,
) where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Pc: PeerRequestChannels + PeerRequestHandler,
    Ns: NodeStateSaver,
    Cl: Cluster,
{
    info!("Node {} Received  vote request {:?}", node_id, request);

    let vote_response = process_vote_request(
        request,
        params.protected_node.clone(),
        params.leader_election_event_tx.clone(),
    );
    let resp_result = {
        trace!("Node {} Sending response {:?}", node_id, vote_response);
        let timeout = Duration::from_secs(1);
        params
            .peer_communicator
            .vote_response_tx(node_id)
            .send_timeout(vote_response, timeout)
    };
    info!("Node {} voted {:?}", node_id, resp_result);
}

pub fn handle_append_entries_request<Log, Rsm, Pc, Ns, Cl>(
    node_id: u64,
    request: AppendEntriesRequest,
    params: &PeerRequestHandlerParams<Log, Rsm, Pc, Ns, Cl>,
) where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Pc: PeerRequestChannels + PeerRequestHandler,
    Ns: NodeStateSaver,
    Cl: Cluster,
{
    let append_entries_response_tx = params.peer_communicator.append_entries_response_tx(node_id);
    trace!("Node {} Received 'Append Entries Request' {:?}", node_id, request);

    let append_entry_response = process_append_entries_request(
        request,
        params.protected_node.clone(),
        params.leader_election_event_tx.clone(),
        params.reset_leadership_watchdog_tx.clone(),
    );

    let send_result = append_entries_response_tx
        .send_timeout(append_entry_response, params.communication_timeout);

    match send_result {
        Ok(_) => {
            trace!("Node {} AppendEntriesResponse sent successfully", node_id);
        }
        Err(err) => {
            error!("Failed append_request processing for Node {} Err: {}", node_id, err);
        }
    }
}

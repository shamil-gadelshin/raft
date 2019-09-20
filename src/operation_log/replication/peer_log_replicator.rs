use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, Sender};

use crate::operation_log::OperationLog;
use crate::state::{Node, NodeStatus, AppendEntriesRequestType, NodeStateSaver};
use crate::communication::peers::{PeerRequestHandler};
use crate::rsm::ReplicatedStateMachine;
use crate::Cluster;


pub struct LogReplicatorParams<Log, Rsm, Pc, Ns, Cl>
	where Log: OperationLog,
		  Rsm: ReplicatedStateMachine,
		  Pc : PeerRequestHandler,
		  Ns : NodeStateSaver,
		  Cl : Cluster{
	pub protected_node : Arc<Mutex<Node<Log, Rsm,Pc, Ns, Cl>>>,
	pub replicate_log_to_peer_rx: Receiver<u64>,
	pub replicate_log_to_peer_tx: Sender<u64>,
	pub communicator : Pc
}


pub fn replicate_log_to_peer<Log, Rsm, Pc, Ns, Cl>(params : LogReplicatorParams<Log, Rsm, Pc, Ns, Cl>,
											   terminate_worker_rx : Receiver<()>)
	where Log: OperationLog,
		  Rsm: ReplicatedStateMachine,
		  Pc : PeerRequestHandler,
		  Ns : NodeStateSaver,
		  Cl : Cluster{
	info!("Peer log replicator worker started");
	loop {
		select!(
			recv(terminate_worker_rx) -> res  => {
                if res.is_err() {
					error!("Abnormal exit for peer log replicator worker");
				}
				break
			},
			recv(params.replicate_log_to_peer_rx) -> peer_id_result => {
				let peer_id = peer_id_result.expect("can receive peer_id from replicate_log_to_peer_rx");
				process_replication_request(&params, peer_id)
			}
		);
	}
	info!("Peer log replicator worker stopped");
}

fn process_replication_request<Log, Rsm, Pc, Ns, Cl>(params: &LogReplicatorParams<Log, Rsm, Pc, Ns, Cl>, peer_id: u64)
	where Log: OperationLog,
		  Rsm: ReplicatedStateMachine,
		  Pc: PeerRequestHandler,
		  Ns: NodeStateSaver,
		  Cl : Cluster {
	trace!("Replicate request for peer {}", peer_id);

	let mut node = params.protected_node.lock().expect("node lock is not poisoned");
	if node.status != NodeStatus::Leader {
		warn!("Obsolete (Not a Leader) replicate log request - Node {} to peer {} ", node.id, peer_id);
		return
	}

	//TODO check for commit index - because of failed consensus - replication can be incorrect

	let next_index = node.get_next_index(peer_id);
	let append_entries_request = node.create_append_entry_request(AppendEntriesRequestType::UpdateNode(peer_id));
	let request_entry_count = append_entries_request.entries.len() as u64;

	if request_entry_count == 0 {
		trace!("Replication: no new entries for Node {}", peer_id);
		return
	}

	let resp_result = params.communicator.send_append_entries_request(peer_id, append_entries_request);
	match resp_result {
		Ok(resp) => {
			if resp.success {
				node.set_next_index(peer_id, next_index + request_entry_count)
			} else {
				if next_index > 1 {
					let modified_next_index = next_index - 1;
					node.set_next_index(peer_id, modified_next_index);
				} else {
					warn!("Unsuccessful replicate log request and next_index <= 1  - Node {} to peer {} ", node.id, peer_id);
				}

				params.replicate_log_to_peer_tx.send(peer_id).expect("can send update peer log request")
			}
		},
		Err(err) => {
			error!("AppendRequest failed for Peer {}:{} ", peer_id, err);
		}
	}
}
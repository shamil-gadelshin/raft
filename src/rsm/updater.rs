use crossbeam_channel::Receiver;

use crate::communication::peers::PeerRequestHandler;
use crate::node::state::{ProtectedNode, NodeStateSaver};
use crate::operation_log::LogEntry;
use crate::operation_log::OperationLog;
use crate::rsm::ReplicatedStateMachine;
use crate::Cluster;

pub struct RsmUpdaterParams<Log, Rsm, Pc, Ns, Cl>
where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Pc: PeerRequestHandler,
    Ns: NodeStateSaver,
    Cl: Cluster,
{
    pub protected_node: ProtectedNode<Log, Rsm, Pc, Ns, Cl>,
    pub commit_index_updated_rx: Receiver<u64>,
}

pub fn update_rsm<Log, Rsm, Pc, Ns, Cl>(
    params: RsmUpdaterParams<Log, Rsm, Pc, Ns, Cl>,
    terminate_worker_rx: Receiver<()>,
) where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Pc: PeerRequestHandler,
    Ns: NodeStateSaver,
    Cl: Cluster,
{
    info!("FSM updater worker started");
    loop {
        select!(
            recv(terminate_worker_rx) -> res  => {
                if res.is_err() {
                    error!("Abnormal exit for FSM updater worker");
                }
                break
            },
            recv(params.commit_index_updated_rx) -> new_commit_index_result => {
                let new_commit_index = new_commit_index_result
                    .expect("valid receive of commit index"); //rsm should be updated

                process_update_fsm_request(&params, new_commit_index)
            }
        );
    }
    info!("FSM updater worker stopped");
}

fn process_update_fsm_request<Log, Rsm, Pc, Ns, Cl>(
    params: &RsmUpdaterParams<Log, Rsm, Pc, Ns, Cl>,
    new_commit_index: u64,
) where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Pc: PeerRequestHandler,
    Ns: NodeStateSaver,
    Cl: Cluster,
{
    let mut node = params
        .protected_node
        .lock()
        .expect("node lock is not poisoned");
    trace!("Update Rsm request for Node {}, Commit index = {}", node.id, new_commit_index);
    loop {
        let last_applied = node.rsm.last_applied_entry_index();

        let mut entry_index = 0;
        let entry_result: Option<LogEntry> = {
            if new_commit_index > last_applied {
                entry_index = last_applied + 1;

                node.log.entry(entry_index)
            } else {
                None
            }
        };

        if let Some(entry) = entry_result {
            let fsm_apply_result = node.rsm.apply_entry(entry);

            if let Err(err) = fsm_apply_result {
                error!("Rsm: 'Apply entry' error. Entry = {}: {}", entry_index, err);
                break;
            }
        } else {
            break;
        }
    }
}

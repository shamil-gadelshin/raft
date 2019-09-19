use std::sync::{Arc};
use std::thread::JoinHandle;
use std::thread;
use crossbeam_channel::{Sender, Receiver};

pub mod peer_consensus_requester;

pub trait QuorumResponse {
    fn get_result(&self) -> bool;
}

pub enum LeaderConfirmationEvent {
    ResetWatchdogCounter
}

#[derive(Clone, Debug)]
pub struct DataEntryContent {
    pub data : Arc<&'static [u8]>
}

#[derive(Clone, Debug)]
pub struct NewClusterConfigurationEntryContent {
    pub new_cluster_configuration: Vec<u64>
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub index: u64,
    pub term: u64,
    pub entry_content: EntryContent
}

#[derive(Clone, Debug)]
pub enum EntryContent {
    AddServer(NewClusterConfigurationEntryContent),
    Data(DataEntryContent),
}

pub fn run_worker_thread<T: Send + 'static, F: Fn(T) + Send + 'static>(worker : F, params : T) -> JoinHandle<()> {
    thread::spawn(move || worker(params))
}

pub struct RaftWorker {
    pub join_handle : JoinHandle<()>,
    pub terminate_worker_tx : Sender<()>
}
pub fn run_worker<T: Send + 'static, F: Fn(T, Receiver<()>) + Send + 'static>(worker : F, params : T) -> RaftWorker {
    let (terminate_worker_tx, terminate_worker_rx): (Sender<()>, Receiver<()>) = crossbeam_channel::unbounded();

    let join_handle = thread::spawn(move|| worker (params, terminate_worker_rx));

    RaftWorker {join_handle, terminate_worker_tx}
}

pub struct RaftWorkerPool {
    workers : Vec<RaftWorker>
}

impl RaftWorkerPool {
    pub fn new(workers : Vec<RaftWorker>) -> RaftWorkerPool {
        RaftWorkerPool {workers}
    }

    pub fn terminate(&self) {
        for worker in &self.workers {
            let send_result = worker.terminate_worker_tx.send(());
            if send_result.is_err() {
                error!("Cannot send termination signal")
            }
        }
    }

    pub fn join(self) {
        for worker in self.workers {
            let join_result = worker.join_handle.join();
            if join_result.is_err() {
                error!("Worker returned an error")
            }
        }
    }
}
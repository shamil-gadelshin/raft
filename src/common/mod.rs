use std::sync::{Arc};
use std::thread::JoinHandle;
use std::thread;
use crossbeam_channel::{Sender, Receiver};

pub mod peer_notifier;

pub trait QuorumResponse {
    fn get_result(&self) -> bool;
}

//TODO downgrade to bool
pub enum LeaderConfirmationEvent {
    ResetWatchdogCounter
}

//TODO separate log & append entry implementations
#[derive(Clone, Debug)]
pub struct DataEntryContent {
    pub data : Arc<&'static [u8]>
}

//TODO separate log & append entry implementations
#[derive(Clone, Debug)]
pub struct AddServerEntryContent {
    pub new_server : u64
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub index: u64,
    pub term: u64,
    pub entry_content: EntryContent
}

#[derive(Clone, Debug)]
pub enum EntryContent {
    AddServer(AddServerEntryContent),
    Data(DataEntryContent),
}

pub fn run_worker_thread<T: Send + 'static, F: Fn(T) + Send + 'static>(worker : F, params : T) -> JoinHandle<()> {
    thread::spawn(move || worker(params))
}

pub struct Worker {
    pub join_handle : JoinHandle<()>,
    pub terminate_worker_tx : Sender<()>
}
pub fn run_worker<T: Send + 'static, F: Fn(T, Receiver<()>) + Send + 'static>(worker : F, params : T) -> Worker {
    let (terminate_worker_tx, terminate_worker_rx): (Sender<()>, Receiver<()>) = crossbeam_channel::unbounded();

    let join_handle = thread::spawn(move|| worker (params, terminate_worker_rx));

    Worker{join_handle, terminate_worker_tx}
}

pub struct WorkerPool {
    workers : Vec<Worker>
}

impl WorkerPool {
    pub fn new(workers : Vec<Worker>) -> WorkerPool {
        WorkerPool{workers}
    }

    pub fn terminate(&self) {
        for worker in &self.workers {
            let send_result = worker.terminate_worker_tx.send(());
            if let Err(_) = send_result {
                error!("Cannot send termination signal")
            }
        }
    }

    pub fn join(self) {
        for worker in self.workers {
            let join_result = worker.join_handle.join();
            if let Err(_) = join_result {
                error!("Worker returned an error")
            }
        }
    }
}
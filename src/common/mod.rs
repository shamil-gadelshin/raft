use crossbeam_channel::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

pub mod peer_consensus_requester;

pub fn run_worker_thread<T, F>(worker: F, params: T) -> JoinHandle<()>
where
    T: Send + 'static,
    F: Fn(T) + Send + 'static,
{
    thread::spawn(move || worker(params))
}

#[derive(Debug)]
pub struct RaftWorker {
    pub join_handle: JoinHandle<()>,
    pub terminate_worker_tx: Sender<()>,
}
pub fn run_worker<T, F>(worker: F, params: T) -> RaftWorker
where
    T: Send + 'static,
    F: Fn(T, Receiver<()>) + Send + 'static,
{
    let (terminate_worker_tx, terminate_worker_rx): (Sender<()>, Receiver<()>) =
        crossbeam_channel::unbounded();

    let join_handle = thread::spawn(move || worker(params, terminate_worker_rx));

    RaftWorker {
        join_handle,
        terminate_worker_tx,
    }
}

pub struct RaftWorkerPool {
    workers: Vec<RaftWorker>,
}

impl RaftWorkerPool {
    pub fn new(workers: Vec<RaftWorker>) -> RaftWorkerPool {
        RaftWorkerPool { workers }
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

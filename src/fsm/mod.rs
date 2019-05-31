use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::time::Duration;
use std::thread::sleep;
use std::thread;

fn start() {
    let (tx, rx): (Sender<NodeStatusEvent>, Receiver<NodeStatusEvent>) = mpsc::channel();
    let node = Node{id : 0, status : NodeStatus::None};
    let mutex_node = Arc::new(Mutex::new(node));

    let fsm_thread_node_mutex = mutex_node.clone();
    let fsm_thread_event_sender = tx.clone();
    let fsm_thread = thread::spawn(move|| fsm_run( fsm_thread_node_mutex, fsm_thread_event_sender, rx));

    let fsm_check_leader_node_mutex = mutex_node.clone();
    let fsm_check_leader_thread_event_sender = tx.clone();
    let fsm_check_leader_thread = thread::spawn(move|| fsm_check_leader_status( fsm_check_leader_node_mutex, fsm_check_leader_thread_event_sender));

    let fsm_check_debug_node_thread = thread::spawn(move|| fsm_debug_node_status( mutex_node.clone()));

    fsm_check_debug_node_thread.join();
    fsm_check_leader_thread.join();
    fsm_thread.join();
}



#[derive(Copy, Clone, Debug)]
enum NodeStatus {
    Candidate,
    Follower,
    Leader,
    None
}

enum NodeStatusEvent {
    PromoteNodeToCandidate,
    PromoteNodeToLeader
}

#[derive(Debug)]
struct Node {
    id : u64,
    status : NodeStatus
}

fn fsm_debug_node_status(mutex_node: Arc<Mutex<Node>>) {
    loop {
        let mut node = mutex_node.lock().expect("lock is poisoned");
        println!("{:?}", node);

        sleep(Duration::from_millis(5000));
    }
}

fn fsm_check_leader_status(mutex_node: Arc<Mutex<Node>>, event_tx : Sender<NodeStatusEvent>) {
    { // mutex_node lock scope
        let mut node = mutex_node.lock().expect("lock is poisoned");

        match node.status {
            NodeStatus::None => {
                let candidate_promotion = NodeStatusEvent::PromoteNodeToCandidate;
                event_tx.send(candidate_promotion);
            },
            _ => ()
        }
    }
// uncomment if no event_tx left
//    loop {
//        sleep(Duration::from_millis(100000));
//    }
}


fn fsm_run(mutex_node: Arc<Mutex<Node>>,event_tx : Sender<NodeStatusEvent>,  event_rx : Receiver<NodeStatusEvent>) {
    println!("ruft FSM started");


    loop {
        let event_result = event_rx.recv();

        let event = event_result.expect("receiving from closed channel");

        match event {
            NodeStatusEvent::PromoteNodeToCandidate => {
                let mut node = mutex_node.lock().expect("lock is poisoned");

                node.status = NodeStatus::Candidate;
                let event_sender = event_tx.clone();
                thread::spawn(move || wait_for_other_leader_notice(event_sender));
            },
            NodeStatusEvent::PromoteNodeToLeader => {
                let mut node = mutex_node.lock().expect("lock is poisoned");

                node.status = NodeStatus::Leader;
            },
        }
    }
}


fn wait_for_other_leader_notice (event_tx : Sender<NodeStatusEvent>) {
    let wait_ms = 7000;
    println!("wait for other leader notice ({:?} ms)...",wait_ms);

    sleep(Duration::from_millis(wait_ms));

    let event_promote_to_leader = NodeStatusEvent::PromoteNodeToLeader;

    event_tx.send(event_promote_to_leader);
}
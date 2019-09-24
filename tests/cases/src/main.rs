#[macro_use] extern crate log;
extern crate env_logger;
extern crate chrono;
extern crate crossbeam_channel;

use std::sync::{Arc};
use std::time::Duration;
use std::collections::HashMap;
use std::io::Write;
use std::thread;

use chrono::prelude::{DateTime, Local};

extern crate raft;
extern crate raft_modules;

use raft::{ClientResponseStatus, ClientRequestHandler, NodeState, NodeWorker, NodeTimings};
use raft::NodeConfiguration;
use raft::NewDataRequest;

use raft_modules::{MemoryRsm, RandomizedElectionTimer, MockNodeStateSaver, ClusterConfiguration};
use raft_modules::MemoryOperationLog;
use raft_modules::{InProcPeerCommunicator};
use raft_modules::NetworkClientCommunicator;


fn init_logger() {
    env_logger::builder()
        .format(|buf, record| {
            let now: DateTime<Local> = Local::now();
            writeln!(buf, "{:5}: {} - {}", record.level(), now.format("%H:%M:%S.%3f").to_string(), record.args())
        })
        .init();
}


fn main() {
    init_logger();

    let node_ids = vec![1, 2];
    let new_node_id = node_ids.last().unwrap() + 1;
    let communication_timeout = Duration::from_millis(500);
    let all_nodes = node_ids.clone();

    let mut communicator = InProcPeerCommunicator::new(all_nodes.clone(), communication_timeout);
    communicator.add_node_communication(new_node_id);

    let mut client_handlers  = HashMap::new(); //: HashMap<u64, ClientRequestHandler>
    let mut node_workers = Vec::new();



    //run initial cluster
    for node_id in all_nodes.clone() {
        let (client_request_handler, node_config) = create_node_configuration(node_id, all_nodes.clone(), communication_timeout,communicator.clone() );

        let node_worker = raft::start_node(node_config);
        node_workers.push(node_worker);

        client_handlers.insert(node_id, client_request_handler);
    }


    thread::sleep(Duration::from_secs(5));


    //find elected leader
    let (leader_client_handler, leader_id) = find_a_leader(client_handlers);

    // run new server
    let new_node_worker = add_new_server(new_node_id, all_nodes, communication_timeout, communicator.clone());
    node_workers.push(new_node_worker);

    //add new server to the cluster
    let add_server_request = raft::AddServerRequest{new_server : new_node_id};
    let resp = leader_client_handler.add_server(add_server_request);
    info!("Add server request sent for Node {}. Response = {:?}", leader_id, resp);

    //add new data to the cluster
    let bytes = "first data".as_bytes();
    let new_data_request = NewDataRequest{data : Arc::new(bytes)};
    let data_resp = leader_client_handler.new_data(new_data_request.clone());
    info!("New Data request sent for Node {}. Response = {:?}", leader_id, data_resp);

    thread::sleep(Duration::from_secs(5));

    thread::spawn(     ||add_thousands_of_data(leader_client_handler));

    thread::sleep(Duration::from_secs(25));
    terminate_workers(node_workers);
}

fn add_new_server(new_node_id: u64, all_nodes: Vec<u64>, communication_timeout: Duration, communicator: InProcPeerCommunicator) -> NodeWorker {
    let (_client_request_handler, new_node_config) = create_node_configuration(new_node_id, all_nodes.clone(), communication_timeout, communicator);
    let node_worker = raft::start_node(new_node_config);

    node_worker
}

fn terminate_workers(node_workers: Vec<NodeWorker>) {
    let mut handles = Vec::new();
    for node_worker in node_workers {
        let handle = node_worker.join_handle;
        handles.push(handle);
        let thread = node_worker.terminate_worker_tx.send(());
        if thread.is_err(){
            panic!("worker panicked!")
        }
    }

    for node_worker in handles {
        let thread = node_worker.join();
        if thread.is_err(){
            panic!("worker panicked!")
        }
    }
}

fn create_node_configuration(node_id: u64, all_nodes: Vec<u64>, communication_timeout: Duration, communicator: InProcPeerCommunicator, )
                             -> (NetworkClientCommunicator, NodeConfiguration<MemoryOperationLog, MemoryRsm, NetworkClientCommunicator, InProcPeerCommunicator, RandomizedElectionTimer, MockNodeStateSaver, ClusterConfiguration>)
{
    let cluster_config =ClusterConfiguration::new(all_nodes);
    let client_request_handler = NetworkClientCommunicator::new(get_address(node_id), node_id, communication_timeout, true);
    let operation_log = MemoryOperationLog::new(cluster_config.clone());
    let config = NodeConfiguration {
        node_state: NodeState {
            node_id,
            current_term: 0,
            vote_for_id: None
        },
        cluster_configuration: cluster_config.clone(),
        peer_communicator: communicator,
        client_communicator: client_request_handler.clone(),
        election_timer: RandomizedElectionTimer::new(1000, 4000),
        operation_log,
        rsm: MemoryRsm::default(),
        state_saver: MockNodeStateSaver::default(),
        timings: NodeTimings::default()
    };

    (client_request_handler, config)
}


pub fn get_address(node_id : u64) -> String{
    format!("127.0.0.1:{}", 50000 + node_id)
}


fn find_a_leader<Cc : ClientRequestHandler>(client_handlers : HashMap<u64, Cc>) -> (Arc<Cc>, u64){
    let bytes = "find a leader".as_bytes();
    let new_data_request = NewDataRequest{data : Arc::new(bytes)};
    for kv in &client_handlers {
        let (k, v) = kv;

        let result = v.new_data(new_data_request.clone());
        if let Ok(resp) = result {
            if let ClientResponseStatus::Ok = resp.status {
                let mut client_handler = Arc::new(v.clone());
                let leader_id = resp.current_leader.expect("can get a leader");

                if *k != leader_id {
                    client_handler = Arc::new(client_handlers[&leader_id].clone());
                }
                return (client_handler, leader_id) ;
            }
        }
    }

    panic!("cannot get a leader!")
}


fn add_thousands_of_data<Cc : ClientRequestHandler + ?Sized + Sync>(client_handler : Arc<Cc>)
{
    //  thread::sleep(Duration::from_secs(7));
    let bytes = "lot of small data".as_bytes();
    let data_request = NewDataRequest{data : Arc::new(bytes)};
    for _count in 1..=10000 {
        let _resp = client_handler.new_data(data_request.clone());
    }
}
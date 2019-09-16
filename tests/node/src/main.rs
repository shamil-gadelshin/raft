#[macro_use] extern crate log;
extern crate env_logger;
extern crate chrono;
extern crate crossbeam_channel;

use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::HashMap;
use std::io::Write;
use std::thread::JoinHandle;
use std::thread;

use chrono::prelude::{DateTime, Local};

extern crate raft;
extern crate raft_modules;

use raft::{ClientResponseStatus, ClientRequestHandler, NodeState};
use raft::ClusterConfiguration;
use raft::NodeConfiguration;
use raft::NewDataRequest;

use raft_modules::{MemoryFsm, RandomizedElectionTimer, MockNodeStateSaver};
use raft_modules::MemoryLogStorage;
use raft_modules::{InProcClientCommunicator};
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
    let new_node_id = 3;
    let communication_timeout = Duration::from_millis(500);
    let main_cluster_configuration = ClusterConfiguration::new(node_ids);

    let mut communicator = InProcPeerCommunicator::new(main_cluster_configuration.get_all(), communication_timeout);
    communicator.add_node_communication(new_node_id);

    let mut client_handlers  = HashMap::new(); //: HashMap<u64, ClientRequestHandler>
    let mut node_threads = Vec::new();
    for node_id in main_cluster_configuration.get_all() {
        let protected_cluster_config = Arc::new(Mutex::new(ClusterConfiguration::new(main_cluster_configuration.get_all())));

        let client_request_handler = NetworkClientCommunicator::new(get_address(node_id), node_id, communication_timeout);
        let config = NodeConfiguration {
            node_state: NodeState {
                node_id,
                current_term: 0,
                vote_for_id : None
            },
            cluster_configuration: protected_cluster_config.clone(),
            peer_communicator: communicator.clone(),
            client_communicator: client_request_handler.clone(),
            election_timer: RandomizedElectionTimer::new(1000, 4000)
        };
        let fsm = MemoryFsm::new(protected_cluster_config.clone());
        let thread_handle = raft::start_node(config, MemoryLogStorage::default(), fsm, MockNodeStateSaver::default());
        node_threads.push(thread_handle);

        client_handlers.insert(node_id, client_request_handler);
    }

    thread::sleep(Duration::from_secs(6));
        let leader_id = find_a_leader(client_handlers.clone());
    println!("Leader: {}", leader_id);

    let protected_cluster_config = Arc::new(Mutex::new(ClusterConfiguration::new(main_cluster_configuration.get_all())));
    let thread_handle = run_add_server_thread_with_delay(communicator.clone(), protected_cluster_config,
                                                         client_handlers.clone(),
                                                         new_node_id);

    node_threads.push(thread_handle);


    let leader_id = find_a_leader(client_handlers.clone());
    add_thousands_of_data(client_handlers.clone(), leader_id);


    for node_thread in node_threads {
        let thread = node_thread.join();
        if thread.is_err(){
            panic!("worker panicked!")
        }
    }
}


pub fn get_address(node_id : u64) -> String{
    format!("127.0.0.1:{}", 50000 + node_id)
}

fn find_a_leader<Cc : ClientRequestHandler>(client_handlers : HashMap<u64, Cc>) -> u64{
    let bytes = "find a leader".as_bytes();
    let new_data_request = NewDataRequest{data : Arc::new(bytes)};
    for kv in client_handlers {
        let (_k, v) = kv;

        let result = v.new_data(new_data_request.clone());
        if let Ok(resp) = result {
            if let ClientResponseStatus::Ok = resp.status {
                return resp.current_leader.expect("can get a leader");
            }
        }
    }

    panic!("cannot get a leader!")
}


fn add_thousands_of_data<Cc : ClientRequestHandler>(client_handlers : HashMap<u64, Cc>, leader_id : u64)
{
    thread::sleep(Duration::from_secs(7));

    let bytes = "find a leader".as_bytes();
    let data_request = NewDataRequest{data : Arc::new(bytes)};
    for _count in 1..=10000 {
        let _resp = client_handlers[&leader_id].new_data(data_request.clone());
    }
}

fn run_add_server_thread_with_delay<Cc : ClientRequestHandler + Clone>(communicator : InProcPeerCommunicator,
                                    protected_cluster_config : Arc<Mutex<ClusterConfiguration>>,
                                    client_handlers : HashMap<u64, Cc>,
                                    new_node_id : u64) -> JoinHandle<()>{
//return;

    let communication_timeout = Duration::from_millis(500);

    thread::sleep(Duration::from_secs(5));

    let new_server_config;
    {
        new_server_config = NodeConfiguration {
            node_state: NodeState {
                node_id: new_node_id,
                current_term: 0,
                vote_for_id : None
            },
            cluster_configuration: protected_cluster_config.clone(),
            peer_communicator: communicator.clone(),
            client_communicator: InProcClientCommunicator::new(new_node_id,communication_timeout),
            election_timer: RandomizedElectionTimer::new(1000, 4000)
        };
    }

    let fsm = MemoryFsm::new(protected_cluster_config.clone());
    let thread_handle = raft::start_node(new_server_config, MemoryLogStorage::default(), fsm, MockNodeStateSaver::default());

    let add_server_request = raft::AddServerRequest{new_server : new_node_id};
    for kv in client_handlers.clone() {
        let (k,v) = kv;

        let resp = v.add_server(add_server_request);

        info!("Add server request sent for NodeId = {:?}. Response = {:?}", k, resp);
    }

    thread::sleep(Duration::from_secs(2));

    let bytes = "first data".as_bytes();
    let new_data_request = NewDataRequest{data : Arc::new(bytes)};
    for kv in client_handlers {
        let (k,v) = kv;

        let resp = v.new_data(new_data_request.clone());

        info!("New Data request sent for NodeId = {:?}. Response = {:?}", k, resp);
    }

    thread_handle
}

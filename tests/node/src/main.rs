#[macro_use] extern crate log;
extern crate env_logger;
extern crate chrono;

mod memory_fsm;
mod memory_log;

use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::HashMap;
use std::io::Write;
use std::thread::JoinHandle;
use std::thread;

use chrono::prelude::{DateTime, Local};

extern crate ruft;

use ruft::{AddServerRequest, InProcClientCommunicator};
use ruft::{InProcPeerCommunicator};
use ruft::ClusterConfiguration;
use ruft::NodeConfiguration;
use ruft::NewDataRequest;

use memory_fsm::MemoryFsm;
use memory_log::MemoryLogStorage;

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

    let mut client_handlers : HashMap<u64, InProcClientCommunicator> = HashMap::new();
    let mut node_threads = Vec::new();
    for node_id in main_cluster_configuration.get_all() {
        let protected_cluster_config = Arc::new(Mutex::new(ClusterConfiguration::new(main_cluster_configuration.get_all())));

        let client_request_handler = InProcClientCommunicator::new(node_id, communication_timeout);
        let config = NodeConfiguration {
            node_id,
            cluster_configuration: protected_cluster_config.clone(),
            peer_communicator: communicator.clone(),
            client_communicator: client_request_handler.clone(),
        };
        let fsm = MemoryFsm::new(protected_cluster_config.clone());
        let thread_handle = ruft::start_node(config, MemoryLogStorage::new(), fsm);
        node_threads.push(thread_handle);

        client_handlers.insert(node_id, client_request_handler);
    }

    let protected_cluster_config = Arc::new(Mutex::new(ClusterConfiguration::new(main_cluster_configuration.get_all())));
    let thread_handle = run_add_server_thread_with_delay(communicator.clone(), protected_cluster_config,
                                                         client_handlers,
                                                         new_node_id);

    node_threads.push(thread_handle);

    for node_thread in node_threads {
        let thread = node_thread.join();
        if thread.is_err(){
            panic!("worker panicked!")
        }
    }
}



fn run_add_server_thread_with_delay(communicator : InProcPeerCommunicator,
                                    protected_cluster_config : Arc<Mutex<ClusterConfiguration>>,
                                    client_handlers : HashMap<u64, InProcClientCommunicator>,
                                    new_node_id : u64) -> JoinHandle<()>{
//return;

    let communication_timeout = Duration::from_millis(500);

    thread::sleep(Duration::from_secs(3));

    let new_server_config;
    {
        new_server_config = NodeConfiguration {
            node_id: new_node_id,
            cluster_configuration: protected_cluster_config.clone(),
            peer_communicator: communicator.clone(),
            client_communicator: InProcClientCommunicator::new(new_node_id,communication_timeout),
        };
    }

    let fsm = MemoryFsm::new(protected_cluster_config.clone());
    let thread_handle = ruft::start_node(new_server_config, MemoryLogStorage::new(), fsm);

    let add_server_request = AddServerRequest{new_server : new_node_id};
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

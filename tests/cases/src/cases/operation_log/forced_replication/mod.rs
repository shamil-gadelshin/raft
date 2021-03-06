use crate::steps;
use crossbeam_channel::{Receiver, Sender};
use raft::{DataEntryContent, EntryContent, LogEntry, OperationLog};
use raft_modules::{ClusterConfiguration, MemoryRsm, RandomizedElectionTimer};
use std::sync::Arc;

mod configuration;
mod custom_operation_log;

use crate::create_node_configuration;

pub fn run() {
    let node_ids = vec![1, 2];
    let new_node_id = node_ids.last().unwrap() + 1;

    let peer_communicator = steps::get_generic_peer_communicator(vec![1, 2, 3]);
    let mut cluster = steps::cluster::start_initial_cluster(
        node_ids,
        peer_communicator,
        configuration::create_different_term_node_configuration_inproc,
    );

    steps::sleep(2);

    //find elected leader
    let leader = cluster.get_node1_leader_by_adding_data_sample();

    //add new data to the cluster
    steps::data::add_data_sample(&leader).expect("add sample successful");

    //create custom operation log
    let (tx, rx): (Sender<Vec<LogEntry>>, Receiver<Vec<LogEntry>>) = crossbeam_channel::unbounded();
    let mut operation_log = custom_operation_log::MemoryOperationLog::new(
        ClusterConfiguration::new(cluster.initial_nodes.clone()),
        tx,
    );

    //add 'existed entry'
    operation_log
        .append_entry(LogEntry {
            term: 1,
            index: 1,
            entry_content: EntryContent::Data(DataEntryContent {
                data: Arc::new(b"bytes"),
            }),
        })
        .expect("append successful");

    // run new server with custom log

    cluster.add_new_server(new_node_id, |node_id, all_nodes, peer_communicator| {
        let cluster_config = ClusterConfiguration::new(all_nodes);
        let (client_request_handler, node_config) = create_node_configuration!(
            node_id,
            cluster_config,
            peer_communicator,
            RandomizedElectionTimer::new(2000, 4000),
            MemoryRsm::new(),
            operation_log.clone()
        );
        let node_worker = raft::start_node(node_config);

        (node_worker, client_request_handler)
    });

    //add new server to the cluster
    steps::data::add_server(&leader, new_node_id);

    steps::sleep(1);

    //add new data to the cluster
    steps::data::add_data_sample(&leader).expect("add sample successful");

    steps::sleep(5);

    //last log contents
    let mut entries = Vec::new();
    loop {
        let res = rx.try_recv();

        if let Ok(val) = res {
            trace!("{:?}", val);
            entries = val;
        } else {
            break;
        }
    }

    assert_eq!(4, entries.len());
    assert_eq!(4, entries[0].term);

    cluster.terminate();
}

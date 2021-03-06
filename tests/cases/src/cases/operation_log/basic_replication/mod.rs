use crate::steps;

use crossbeam_channel::{Receiver, Sender};
use raft::LogEntry;
use raft_modules::{ClusterConfiguration, MemoryRsm, RandomizedElectionTimer};

mod custom_operation_log;

use crate::create_node_worker;

pub fn run() {
    let node_ids = vec![1, 2];
    let new_node_id = node_ids.last().unwrap() + 1;

    let peer_communicator = steps::get_generic_peer_communicator(vec![1, 2, 3]);
    let mut cluster = steps::cluster::start_initial_cluster(
        node_ids,
        peer_communicator,
        steps::create_generic_node_inproc,
    );

    steps::sleep(2);

    //find elected leader
    let leader = cluster.get_node1_leader_by_adding_data_sample();

    //add new data to the cluster
    steps::data::add_data_sample(&leader).expect("add sample successful");

    let (tx, rx): (Sender<LogEntry>, Receiver<LogEntry>) = crossbeam_channel::unbounded();
    let operation_log = custom_operation_log::MemoryOperationLog::new(
        ClusterConfiguration::new(cluster.initial_nodes.clone()),
        tx,
    );
    // run new server
    cluster.add_new_server(new_node_id, |node_id, all_nodes, peer_communicator| {
        create_node_worker!(
            node_id,
            ClusterConfiguration::new(all_nodes),
            peer_communicator,
            RandomizedElectionTimer::new(2000, 4000),
            MemoryRsm::new(),
            operation_log.clone()
        )
    });

    //add new server to the cluster
    steps::data::add_server(&leader, new_node_id);

    steps::sleep(1);
    //add new data to the cluster
    steps::data::add_data_sample(&leader).expect("add sample successful");

    steps::sleep(5);

    let mut entry_count = 0;
    loop {
        let res = rx.try_recv();

        if let Ok(val) = res {
            entry_count += 1;
            trace!("{:?}", val);
        } else {
            // println!("{:?}", res);
            break;
        }
    }

    assert_eq!(4, entry_count);

    cluster.terminate();
}

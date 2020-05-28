use crate::steps;

use crossbeam_channel::{Receiver, Sender};
use raft::DataEntryContent;
use raft_modules::RandomizedElectionTimer;

use crate::create_node_configuration_in_proc;
mod custom_rsm;

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

    let (tx, rx): (Sender<DataEntryContent>, Receiver<DataEntryContent>) =
        crossbeam_channel::unbounded();
    let rsm = custom_rsm::MemoryRsm::new(tx);
    // run new server
    cluster.add_new_server(new_node_id, |node_id, all_nodes, peer_communicator| {
        let election_timer = RandomizedElectionTimer::new(2000, 4000);
        let (client_request_handler, node_config) = create_node_configuration_in_proc!(
            node_id,
            all_nodes,
            peer_communicator,
            election_timer,
            rsm.clone()
        );
        let node_worker = raft::start_node(node_config);

        (node_worker, client_request_handler)
    });

    //add new server to the cluster
    steps::data::add_server(&leader, new_node_id);

    steps::sleep(1);
    //add new data to the cluster
    //	steps::data::add_data_sample(&leader).expect("add sample successful");
    steps::data::add_ten_thousands_data_samples(leader);

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

    assert_eq!(10002, entry_count);

    cluster.terminate();
}

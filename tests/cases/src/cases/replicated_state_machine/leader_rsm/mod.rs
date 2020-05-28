use crate::steps;

use crossbeam_channel::{Receiver, Sender};
use raft::DataEntryContent;
use raft_modules::FixedElectionTimer;

use crate::create_node_configuration_in_proc;
mod custom_rsm;

pub fn run() {
    let node_ids = vec![1];

    let peer_communicator = steps::get_generic_peer_communicator(vec![1, 2, 3]);

    let (tx, rx): (Sender<DataEntryContent>, Receiver<DataEntryContent>) =
        crossbeam_channel::unbounded();
    let rsm = custom_rsm::MemoryRsm::new(tx);
    let cluster = steps::cluster::start_initial_cluster(
        node_ids,
        peer_communicator,
        |node_id, all_nodes, peer_communicator| {
            let election_timer = FixedElectionTimer::new(1000);
            let (client_request_handler, node_config) = create_node_configuration_in_proc!(
                node_id,
                all_nodes,
                peer_communicator,
                election_timer,
                rsm.clone(),
            );
            let node_worker = raft::start_node(node_config);

            (node_worker, client_request_handler)
        },
    );

    steps::sleep(2);

    //find elected leader
    let leader = cluster.get_node1_leader_by_adding_data_sample();

    steps::data::add_ten_thousands_data_samples(leader);

    steps::sleep(1);

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

    assert_eq!(10001, entry_count);

    cluster.terminate();
}

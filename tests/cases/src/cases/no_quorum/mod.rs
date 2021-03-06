mod custom_peer_communicator;

use crate::steps;
use raft::ClientResponseStatus;

pub fn run() {
    let node_ids = vec![1, 2, 3, 4];

    let peer_communicator = custom_peer_communicator::InProcPeerCommunicator::new(
        vec![1, 2, 3, 4],
        steps::get_peers_communication_timeout(),
    );
    let cluster = steps::cluster::start_initial_cluster(
        node_ids,
        peer_communicator,
        steps::create_generic_node_inproc,
    );

    steps::sleep(4);

    //find elected leader
    let leader = cluster.find_a_leader_by_adding_data_sample();
    assert_eq!(1, leader.id);

    //add new data to the cluster
    let resp = steps::data::add_data_sample(&leader).expect("add sample successful");
    assert_eq!(ClientResponseStatus::NoQuorum, resp.status);

    steps::sleep(1);

    cluster.terminate();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_noquorum() {
        crate::cases::no_quorum::run()
    }
}

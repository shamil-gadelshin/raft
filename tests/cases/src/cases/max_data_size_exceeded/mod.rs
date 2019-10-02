use crate::steps;
use raft::ClientResponseStatus;

mod configuration;

pub fn run() {
    let node_ids = vec![1];

    let peer_communicator = steps::get_generic_peer_communicator(vec![1, 2]);
    let cluster = steps::cluster::start_initial_cluster(
        node_ids,
        peer_communicator.clone(),
        configuration::create_custom_node_inproc,
    );

    steps::sleep(5);

    //find elected leader
    let leader = cluster.find_a_leader_by_adding_data_sample();

    //add new tiny data to the cluster
    let tiny_data_res = steps::data::add_tiny_data_sample(&leader).expect("add sample successful");
    assert_eq!(ClientResponseStatus::Ok, tiny_data_res.status);

    //add new  small data to the cluster
    let small_data_res = steps::data::add_data_sample(&leader).expect("add sample successful");
    assert_eq!(ClientResponseStatus::Error, small_data_res.status);
    info!("{:?}", small_data_res);

    steps::sleep(1);

    cluster.terminate();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_max_data_size_exceeded() {
        crate::cases::max_data_size_exceeded::run()
    }
}

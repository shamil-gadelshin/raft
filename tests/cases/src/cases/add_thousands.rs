use crate::steps;

pub fn run() {
    let node_ids = vec![1, 2];
    let new_node_id = node_ids.last().unwrap() + 1;

    let peer_communicator = steps::get_generic_peer_communicator(vec![1, 2, 3]);
    let mut cluster = steps::cluster::start_initial_cluster(
        node_ids,
        peer_communicator,
        steps::create_generic_node_inproc,
    );

    steps::sleep(5);

    //find elected leader
    let leader = cluster.find_a_leader_by_adding_data_sample();

    // run new server
    cluster.add_new_server(new_node_id, steps::create_generic_node_inproc);

    //add new server to the cluster
    steps::data::add_server(&leader, new_node_id);

    //add new data to the cluster
    steps::data::add_data_sample(&leader).expect("add sample successful");

    steps::sleep(5);

    steps::data::add_ten_thousands_data_samples(leader);

    cluster.terminate();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_add_thousands() {
        crate::cases::add_thousands::run()
    }
}

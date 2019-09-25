use crate::steps;

pub fn run() {

		let node_ids = vec![1, 2];
		let new_node_id = node_ids.last().unwrap() + 1;

		let peer_communicator = steps::peer_communicator::get_peer_communicator( vec![1, 2, 3]);
		let mut cluster = steps::cluster::start_initial_cluster(node_ids, peer_communicator.clone(), steps::create_node_with_network);

		steps::sleep(5);

		//find elected leader
		let leader = cluster.find_a_leader();

		// run new server
		cluster.add_new_server(new_node_id, steps::create_node_with_network);


		//add new server to the cluster
		steps::data::add_server(&leader, new_node_id);

		//add new data to the cluster
		steps::data::add_data_sample(&leader.clone()).expect("add sample successful");

		steps::sleep(5);

		cluster.terminate();

}
use crate::steps;

pub fn run() {

	let node_ids = vec![1];
	let new_node_id = node_ids.last().unwrap() + 1;

	let peer_communicator = steps::peer_communicator::get_peer_communicator( vec![1, 2]);
	let mut cluster = steps::cluster::start_initial_cluster(node_ids, peer_communicator.clone(), steps::create_node_inproc);

	steps::sleep(5);

	//find elected leader
	let leader = cluster.find_a_leader_by_adding_data_sample();

	//add new data to the cluster
	steps::data::add_data_sample(&leader).expect("add sample successful");

	// run new server
	cluster.add_new_server(new_node_id, steps::create_node_inproc);

	//add new server to the cluster
	steps::data::add_server(&leader, new_node_id);

	steps::sleep(5);
	//add new data to the cluster
	steps::data::add_data_sample(&leader).expect("add sample successful");

	steps::sleep(2);


	cluster.terminate();
}
use raft::{NewDataRequest, ClientRequestHandler};
use std::sync::Arc;
use crate::steps::cluster::Leader;
use std::thread;

pub fn add_data_sample<Cc : ClientRequestHandler>(leader: &Leader<Cc>) {
	let bytes = "first data".as_bytes();
	let new_data_request = NewDataRequest{data : Arc::new(bytes)};
	let data_resp = leader.client_handler.new_data(new_data_request.clone());
	info!("Add server request sent for Node {}. Response = {:?}", leader.id, data_resp);
}

pub fn add_server<Cc : ClientRequestHandler>(leader: &Leader<Cc>, new_node_id : u64) {
	let add_server_request = raft::AddServerRequest{new_server : new_node_id};
	let resp = leader.client_handler.add_server(add_server_request);
	info!("Add server request sent for Node {}. Response = {:?}", leader.id, resp);
}

pub fn add_thousands_data_samples<Cc : ClientRequestHandler>(leader : Leader<Cc>) {
	fn add_thousands_of_data<Cc : ClientRequestHandler + ?Sized + Sync>(client_handler : Arc<Cc>)
	{
		//  thread::sleep(Duration::from_secs(7));
		let bytes = "lot of small data".as_bytes();
		let data_request = NewDataRequest{data : Arc::new(bytes)};
		for _count in 1..=10000 {
			let _resp = client_handler.new_data(data_request.clone());
		}
	}

	thread::spawn(     ||add_thousands_of_data(leader.client_handler));
}
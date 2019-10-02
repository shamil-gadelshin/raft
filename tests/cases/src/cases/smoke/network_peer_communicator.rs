use raft_modules::{NetworkPeerCommunicator, StaticTableServiceDiscovery};
use crate::steps::get_peers_communication_timeout;
use std::collections::HashMap;

pub fn get_network_peer_communicator(node_id: u64) -> NetworkPeerCommunicator<StaticTableServiceDiscovery>{
	let table : HashMap<u64, String>  = [
		(1, get_peer_requests_address(1)),
		(2, get_peer_requests_address(2)),
		(3, get_peer_requests_address(3))]
		.iter().cloned().collect();

	let service_discovery = StaticTableServiceDiscovery::new(table);

	let host = get_peer_requests_address(node_id);

	NetworkPeerCommunicator::new(host, node_id,get_peers_communication_timeout(), true, service_discovery)
}


pub fn get_peer_requests_address(node_id : u64) -> String{
	format!("127.0.0.1:{}", 60000 + node_id)
}

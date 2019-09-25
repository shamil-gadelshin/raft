use raft_modules::InProcPeerCommunicator;

pub fn get_peer_communicator(nodes : Vec<u64>) -> InProcPeerCommunicator{
	 InProcPeerCommunicator::new(nodes, super::get_peers_communication_timeout())
}
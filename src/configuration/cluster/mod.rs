use std::collections::HashMap;

#[derive(Clone, Debug)]
//TODO save changes to cluster configuration
pub struct ClusterConfiguration{
    nodes_id_map : HashMap<u64, ()>,
}

impl ClusterConfiguration {
    pub fn get_quorum_size(&self) -> u32 {
        let node_count = self.nodes_id_map.len() as u32;

        if node_count == 0 {
            panic!("Cannot calculate quorum size: node_count = 0")
        }

        let half = node_count / 2;

        half + 1 //majority
    }

    pub fn get_peers(&self, node_id : u64) -> Vec<u64>{
        let mut peer_ids = self.get_all_nodes();
        peer_ids.retain(|&x| x != node_id);

        peer_ids
    }

    pub fn new(peers : Vec<u64>) -> ClusterConfiguration {
        let mut cluster_config = ClusterConfiguration{nodes_id_map : HashMap::new()};

        for node in peers {
            cluster_config.add_peer(node);
        }

        cluster_config
    }

    pub fn add_peer(&mut self, peer : u64) {
        if self.nodes_id_map.contains_key(&peer) {
            warn!("Cluster configuration - add duplicate peer:{}", peer)
        }
        self.nodes_id_map.insert(peer, ());
    }

    pub fn get_all_nodes(&self) -> Vec<u64> {
        self.nodes_id_map.keys().cloned().collect()
    }
}

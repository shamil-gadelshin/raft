use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ClusterConfiguration{
    nodes_id_map : HashMap<u64, bool>,
}

impl ClusterConfiguration {
    pub fn get_quorum_size(&self) -> u32 {
        self.nodes_id_map.len() as u32
    }

    pub fn get_peers(&self, node_id : u64) -> Vec<u64>{
        let mut peer_ids = self.get_all();
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
        self.nodes_id_map.insert(peer, true);
    }

    pub fn get_all(&self)-> Vec<u64> {
        let key_vector =  self.nodes_id_map.keys().map(|key| *key).collect();

        key_vector
    }
}
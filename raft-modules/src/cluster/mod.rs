use std::collections::HashMap;
use raft::Cluster;
use std::sync::{Mutex, Arc};

#[derive(Clone, Debug)]
pub struct ClusterConfiguration{
    cluster : Arc<Mutex<ClusterConfigurationInternal>>
}

impl Cluster for ClusterConfiguration{
    fn get_quorum_size(&self) -> u32 {
        let cluster = self.cluster.lock().expect("cluster lock is not poisoned");

        cluster.get_quorum_size()
    }

    fn get_all_nodes(&self) -> Vec<u64> {
        let cluster = self.cluster.lock().expect("cluster lock is not poisoned");

        cluster.get_all_nodes()
    }

    fn get_peers(&self, node_id: u64) -> Vec<u64> {
        let cluster = self.cluster.lock().expect("cluster lock is not poisoned");

        cluster.get_peers(node_id)
    }
}

#[derive(Clone, Debug)]
//TODO save changes to cluster configuration (persistence)
struct ClusterConfigurationInternal{
    nodes_id_map : HashMap<u64, ()>,
}

impl Cluster for ClusterConfigurationInternal {
    fn get_quorum_size(&self) -> u32 {
        let node_count = self.nodes_id_map.len() as u32;

        if node_count == 0 {
            panic!("Cannot calculate quorum size: node_count = 0")
        }

        let half = node_count / 2;

        half + 1 //majority
    }

    fn get_all_nodes(&self) -> Vec<u64> {
        self.nodes_id_map.keys().cloned().collect()
    }

    fn get_peers(&self, node_id : u64) -> Vec<u64>{
        let mut peer_ids = self.get_all_nodes();
        peer_ids.retain(|&x| x != node_id);

        peer_ids
    }
}

impl ClusterConfigurationInternal {
    pub fn new(peers : Vec<u64>) -> ClusterConfigurationInternal {
        let mut cluster_config = ClusterConfigurationInternal{nodes_id_map : HashMap::new()};

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
}

impl ClusterConfiguration {
    pub fn new(peers : Vec<u64>) -> ClusterConfiguration {
        let cluster_config = ClusterConfigurationInternal::new(peers);

        ClusterConfiguration{cluster: Arc::new(Mutex::new(cluster_config))}
    }

    pub fn add_peer(&mut self, peer : u64) {
        let mut cluster = self.cluster.lock().expect("cluster lock is not poisoned");

        cluster.add_peer(peer);
    }
}
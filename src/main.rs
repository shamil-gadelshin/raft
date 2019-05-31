mod fsm;

use std::thread;
use fsm::communication::{VoteRequest, VoteResponse};
use fsm::communication::{InProcNodeCommunicator};
use std::collections::HashMap;

#[macro_use]
extern crate crossbeam_channel;

use crossbeam_channel::{Sender, Receiver};

fn main() {
    let node_ids = vec![1,2];

    let mut request_tx_channels = HashMap::new();
    let mut request_rx_channels = HashMap::new();
    let mut response_tx_channels = HashMap::new();
    let mut response_rx_channels = HashMap::new();


    for node_id in node_ids.clone() {
         let (request_tx, request_rx): (Sender<VoteRequest>, Receiver<VoteRequest>) = crossbeam_channel::unbounded();
         let (response_tx, response_rx): (Sender<VoteResponse>, Receiver<VoteResponse>) = crossbeam_channel::unbounded();

        request_tx_channels.insert(node_id, request_tx);
        request_rx_channels.insert(node_id, request_rx);
        response_tx_channels.insert(node_id, response_tx);
        response_rx_channels.insert(node_id, response_rx);
    }

    let communicator = InProcNodeCommunicator{request_channels_tx : request_tx_channels, response_channels_tx : response_tx_channels};

    for node_id in node_ids.clone() {
        let mut peer_ids = node_ids.clone();
        peer_ids.retain(|&x| x != node_id);

    //    let (request_tx, response_rx): (Sender<VoteRequest>, Receiver<VoteRequest>) = mpsc::channel();

        let request_rx = request_rx_channels.remove(&node_id).unwrap();
        let response_rx = response_rx_channels.remove(&node_id).unwrap();

        let config = fsm::NodeConfiguration{
            node_id,
            peers_id_list: peer_ids,
            quorum_size : node_ids.len() as u32,
            request_rx_channel : request_rx,
            response_rx_channel : response_rx,
            communicator : communicator.clone()
        };
        thread::spawn(move || fsm::start(config));

    }

    thread::park(); //TODO -  to join
}

/*
TODO:
- logging
- crossbeam?rayon threading
- crossbeam msmp channels
- futures?
- tests
- speed & memory profiling
- identity - libp2p
- generic identity?
- tarpc

Features:
- log replication
.memory snapshot
.file snapshot
.empty snapshot?
- membership changes
- leader election
*/

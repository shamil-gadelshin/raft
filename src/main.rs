use std::thread;
use std::collections::HashMap;


#[macro_use]
extern crate crossbeam_channel;
use crossbeam_channel::{Sender, Receiver};

extern crate chrono;


mod leadership;
use crate::communication::{VoteRequest, VoteResponse, AppendEntriesRequest};
use crate::communication::{InProcNodeCommunicator};

mod communication;
mod log_replication;
mod runner;


fn main() {
    let node_ids = vec![1,2];

    let mut vote_request_tx_channels = HashMap::new();
    let mut vote_request_rx_channels = HashMap::new();
    let mut vote_response_tx_channels = HashMap::new();
    let mut vote_response_rx_channels = HashMap::new();
    let mut append_entries_request_tx_channels = HashMap::new();
    let mut append_entries_request_rx_channels = HashMap::new();


    for node_id in node_ids.clone() {
         let (vote_request_tx, vote_request_rx): (Sender<VoteRequest>, Receiver<VoteRequest>) = crossbeam_channel::unbounded();
         let (vote_response_tx, vote_response_rx): (Sender<VoteResponse>, Receiver<VoteResponse>) = crossbeam_channel::unbounded();
         let (append_entries_request_tx, append_entries_request_rx): (Sender<AppendEntriesRequest>, Receiver<AppendEntriesRequest>) = crossbeam_channel::unbounded();

        vote_request_tx_channels.insert(node_id, vote_request_tx);
        vote_request_rx_channels.insert(node_id, vote_request_rx);
        vote_response_tx_channels.insert(node_id, vote_response_tx);
        vote_response_rx_channels.insert(node_id, vote_response_rx);
        append_entries_request_tx_channels.insert(node_id, append_entries_request_tx);
        append_entries_request_rx_channels.insert(node_id, append_entries_request_rx);
    }

    let communicator = InProcNodeCommunicator{
        vote_request_channels_tx: vote_request_tx_channels,
        vote_response_channels_tx: vote_response_tx_channels,
        append_entries_request_channels_tx : append_entries_request_tx_channels
    };

    for node_id in node_ids.clone() {
        let mut peer_ids = node_ids.clone();
        peer_ids.retain(|&x| x != node_id);

    //    let (request_tx, response_rx): (Sender<VoteRequest>, Receiver<VoteRequest>) = mpsc::channel();

        let vote_request_rx = vote_request_rx_channels.remove(&node_id).unwrap();
        let vote_response_rx = vote_response_rx_channels.remove(&node_id).unwrap();
        let append_entries_rx = append_entries_request_rx_channels.remove(&node_id).unwrap();

        let config = runner::NodeConfiguration{
            node_id,
            peers_id_list: peer_ids,
            quorum_size : node_ids.len() as u32,
            vote_request_rx_channel: vote_request_rx,
            vote_response_rx_channel: vote_response_rx,
            append_entries_rx_channel: append_entries_rx,
            communicator : communicator.clone()
        };
        thread::spawn(move || runner::start(config));

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
- consider replacing mutex with cas
- check channels overflow

Features:
- log replication
.memory snapshot
.file snapshot
.empty snapshot?
- membership changes
- leader election
*/

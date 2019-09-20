use std::result::Result;
use std::ops::{Fn};
use std::string::*;
use rayon::prelude::*;

use crate::errors;
use super::QuorumResponse;
use crate::errors::RaftError;


pub fn request_peer_consensus<Req, Resp, Requester>(request: Req,
                                                    node_id : u64,
                                                    peers : Vec<u64>,
                                                    quorum: Option<u32>,
                                                    requester : Requester) -> Result<bool, RaftError>
where Requester: Fn(u64, Req) ->Result<Resp, RaftError>,
      Req: Clone,
      Resp: QuorumResponse {
    if peers.is_empty() {
        return Ok(true)
    }

    let responses = get_responses_from_peer(request, peers, requester);

    //quorum required
    if let Some(quorum_size) = quorum {
        let mut votes = 1; //self voted already
        let mut errors = Vec::new();
        for response in responses {
            match response {
                Ok(peer_resp) => {
                    if peer_resp.get_result() {
                        votes += 1;
                    }

                    if votes >= quorum_size {
                        trace!("Node {} gathered quorum for request", node_id);
                        return Ok(true);
                    }
                },
                Err(err) => {
                    errors.push(err);
                }
            }
        }

        info!("Node {}: cannot get quorum for request. Vote count: {:?}", node_id, votes);
        if !errors.is_empty() && votes == 1 { //no responses
            return errors::new_multiple_err("Cannot get quorum for request".to_string(), errors)
        }
        return Ok(false)
    }

    Ok(true)
}

fn get_responses_from_peer<Req, Resp, Requester>(request: Req, peers: Vec<u64>, requester: Requester) -> Vec<Result<Resp, RaftError>>
    where Requester: Fn(u64, Req) -> Result<Resp, RaftError>,
          Req: Clone,
          Resp: QuorumResponse {

//TODO communicator result & timeout handling
//TODO rayon parallel-foreach


    let mut responses = Vec::new();
    for peer_id in peers {

        let resp = requester(peer_id, request.clone());
        responses.push(resp);
    }

//    let mut resp_iter = peers.into_par_iter().map(|peer_id| {
//        let resp = requester(peer_id, request.clone());
//
//        resp
//    });
//
//    let responses: Vec<_> = resp_iter.collect();
    responses
}

use std::result::Result;
use std::ops::{Fn};
use std::string::*;
use std::error::Error;

use crate::errors;
use super::QuorumResponse;



//TODO add proper error handling
//TODO rename
pub fn notify_peers<Req, Resp, Requester>(request: Req,
    node_id : u64,
    peers : Vec<u64>,
    quorum: Option<u32>,
    requester : Requester) -> Result<bool,Box<Error>>
where Requester: Fn(u64, Req) ->Result<Resp,Box<Error>>,
      Req: Clone,
      Resp: QuorumResponse {
    if peers.is_empty() {
        return Ok(true)
    }

    //TODO communicator timeout handling
    //TODO rayon parallel-foreach
    let mut responses = Vec::new();
    for peer_id in peers {
        let resp = requester(peer_id, request.clone());
        responses.push(resp);
    }

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
                        trace!("Node {:?} gathered quorum for request", node_id);
                        return Ok(true);
                    }
                },
                Err(err) => {
                    errors.push(err);
                }
            }
        }

        info!("Node {:?}: cannot get quorum for request. Vote count: {:?}", node_id, votes);
        if !errors.is_empty() && votes == 1 { //no responses
            return errors::new_multiple_err("Cannot get quorum for request".to_string(), errors)
        }
        return Ok(false)
    }

    Ok(true)
}

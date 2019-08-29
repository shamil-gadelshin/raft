use std::result::Result;
use std::ops::{Fn};
use std::string::*;

use super::QuorumResponse;

//TODO add proper error handling
pub fn notify_peers<Req, Resp, Requester>(request: Req,
    node_id : u64,
    peers : Vec<u64>,
    quorum: Option<u32>,
    requester : Requester) -> Result<(), &'static str>
where Requester: Fn(u64, Req) ->Result<Resp, &'static str>,
      Req: Clone,
      Resp: QuorumResponse{
    if !peers.is_empty() {

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
            let mut errors = String::from("No quorum");
            for response in responses {
                match response {
                    Ok(peer_resp) => {
                        if peer_resp.get_result() {
                            votes += 1;
                        }

                        if votes >= quorum_size {
                            trace!("Node {:?} gathered quorum for request", node_id);
                            return Ok(());
                        }
                    },
                    Err(err) => {
                        errors.push_str(";");
                        errors.push_str(err);
                    }
                }
            }

            info!("Node {:?}: cannot get quorum for request. Vote count: {:?}", node_id, votes);
            return Err(Box::leak(errors.into_boxed_str()))
        }
    }

    Ok(())
}

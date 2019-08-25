use crossbeam_channel::{Sender, Receiver};
use std::marker::PhantomData;

use crate::core::*;
use std::time::Duration;

#[derive(Clone)]
pub struct DuplexChannel<Request, Response> {
    pub request_tx: Sender<Request>,
    pub request_rx: Receiver<Request>,
    pub response_tx: Sender<Response>,
    pub response_rx: Receiver<Response>,
}

impl <Request, Response> DuplexChannel<Request, Response> {
    pub fn new() -> DuplexChannel<Request, Response> {
        let (request_tx, request_rx): (Sender<Request>, Receiver<Request>) = crossbeam_channel::bounded(0);
        let (response_tx, response_rx): (Sender<Response>, Receiver<Response>) = crossbeam_channel::bounded(0);

        let duplex_channel = DuplexChannel{
            request_tx,
            request_rx,
            response_tx,
            response_rx
        };

        duplex_channel
    }

    pub fn get_request_rx(&self) -> Receiver<Request> {
        self.request_rx.clone()
    }

    pub fn get_request_tx(&self) -> Sender<Request> {
        self.request_tx.clone()
    }

    pub fn get_response_tx(&self) -> Sender<Response> {
        self.response_tx.clone()
    }
    pub fn get_response_rx(&self) -> Receiver<Response> {
        self.response_rx.clone()
    }

    //TODO change result error type
    pub fn send_request(&self, request: Request) -> Result<Response, &'static str> {
        let timeout = Duration::from_millis(500);

        let send_result = self.request_tx.send_timeout(request, timeout);
        if let Err(err) = send_result {
            return Err("Cannot send request. Timeout.")
        }

        let receive_result = self.response_rx.recv_timeout(timeout);
        if let Err(err) = receive_result {
            return Err("Cannot receive from response_rx")
        }
        if let Ok(resp) = receive_result {
            let response = resp;

            return Ok(response);
        }

        panic!("invalid request-response sequence");
    }
}


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
//        print_event( format!("Add server request {:?}", request));

        let timeout = crossbeam_channel::after(Duration::new(1,0));
        select!(
            recv(timeout) -> _  => {
                return Err("Send request timeout")
            },
            send(self.request_tx, request) -> res => {
                if let Err(err) = res {
                    return Err("Cannot send request")
                }
            },
        );

        select!(
            recv(timeout) -> _  => {
                return Err("Receive response timeout")
            },
            recv(self.response_rx) -> res => {
                if let Err(err) = res {
                    return Err("Cannot receive from response_rx")
                }
                if let Ok(resp) = res {
                    let response = resp;

                    return Ok(response);
                }
            },
        );

        panic!("invalid request-response sequence");
    }
}


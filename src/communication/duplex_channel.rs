use crossbeam_channel::{Sender, Receiver};

use std::time::Duration;

use crate::errors;
use std::error::Error;

//TODO add name for logging purpose
#[derive(Clone, Debug)]
pub struct DuplexChannel<Request, Response> {
    name : String,
    timeout_duration: Duration,
    pub request_tx: Sender<Request>,
    pub request_rx: Receiver<Request>,
    pub response_tx: Sender<Response>,
    pub response_rx: Receiver<Response>,
}

impl <Request, Response> DuplexChannel<Request, Response>
where Request: Send + 'static{
    pub fn new(name : String, timeout_duration: Duration) -> DuplexChannel<Request, Response> {
        let (request_tx, request_rx): (Sender<Request>, Receiver<Request>) = crossbeam_channel::bounded(0);
        let (response_tx, response_rx): (Sender<Response>, Receiver<Response>) = crossbeam_channel::bounded(0);

        DuplexChannel{
            timeout_duration,
            name,
            request_tx,
            request_rx,
            response_tx,
            response_rx
        }
    }

    pub fn get_request_rx(&self) -> Receiver<Request> {
        self.request_rx.clone()
    }
    #[allow(dead_code)]
    pub fn get_request_tx(&self) -> Sender<Request> { self.request_tx.clone() }

    pub fn get_response_tx(&self) -> Sender<Response> {
        self.response_tx.clone()
    }
    #[allow(dead_code)]
    pub fn get_response_rx(&self) -> Receiver<Response> { self.response_rx.clone() }

    //TODO consider & change result error type
    pub fn send_request(&self, request: Request) -> Result<Response, Box<Error>> {
        let send_result = self.request_tx.send_timeout(request, self.timeout_duration);
        if let Err(err) = send_result {
            return
               errors::new_err( format!("Cannot send request. Channel : {} ", self.name),  Some(Box::new(err)))

        }

        let receive_result = self.response_rx.recv_timeout(self.timeout_duration);
        if let Err(err) = receive_result {
            return errors::new_err(format!("Cannot receive response. Channel : {}", self.name), Some(Box::new(err)))
        }
        if let Ok(resp) = receive_result {
            let response = resp;

            return Ok(response);
        }

        unreachable!("invalid request-response sequence");
    }
}


use crossbeam_channel::{Receiver, Sender};

use std::time::Duration;

use raft::{new_err, RaftError};

/// Create abstraction for the dual-end communication via channels.
#[derive(Clone, Debug)]
pub struct DuplexChannel<Request, Response> {
    name: String,
    timeout_duration: Duration,

    /// Sender channel for the request.
    pub request_tx: Sender<Request>,

    /// Receiver channel for the request.
    pub request_rx: Receiver<Request>,

    /// Sender channel for the response.
    pub response_tx: Sender<Response>,

    /// Receiver channel for the response.
    pub response_rx: Receiver<Response>,
}

impl<Request, Response> DuplexChannel<Request, Response>
where
    Request: Send + 'static,
{
    /// Creates new DuplexChannel with the name and communication timeout on recv's and send's.
    pub fn new(name: String, timeout_duration: Duration) -> DuplexChannel<Request, Response> {
        let (request_tx, request_rx): (Sender<Request>, Receiver<Request>) =
            crossbeam_channel::bounded(0);
        let (response_tx, response_rx): (Sender<Response>, Receiver<Response>) =
            crossbeam_channel::bounded(0);

        DuplexChannel {
            timeout_duration,
            name,
            request_tx,
            request_rx,
            response_tx,
            response_rx,
        }
    }

     /// Returns the receiver channel for the request.
    pub fn request_rx(&self) -> Receiver<Request> {
        self.request_rx.clone()
    }

    /// Returns the sender channel for the request.
    #[allow(dead_code)]
    pub fn request_tx(&self) -> Sender<Request> {
        self.request_tx.clone()
    }

    /// Returns the sender channel for the response.
    pub fn response_tx(&self) -> Sender<Response> {
        self.response_tx.clone()
    }

    /// Returns the receiver channel for the response.
    #[allow(dead_code)]
    pub fn response_rx(&self) -> Receiver<Response> {
        self.response_rx.clone()
    }

    /// Sends request and gets response via channels.
    pub fn send_request(&self, request: Request) -> Result<Response, RaftError> {
        let send_result = self.request_tx.send_timeout(request, self.timeout_duration);
        if let Err(err) = send_result {
            return new_err(
                format!("Cannot send request. Channel : {} ", self.name),
                err.to_string(),
            );
        }

        let receive_result = self.response_rx.recv_timeout(self.timeout_duration);
        if let Err(err) = receive_result {
            return new_err(
                format!("Cannot receive response. Channel : {}", self.name),
                err.to_string(),
            );
        }
        if let Ok(resp) = receive_result {
            let response = resp;

            return Ok(response);
        }

        unreachable!("invalid request-response sequence");
    }
}

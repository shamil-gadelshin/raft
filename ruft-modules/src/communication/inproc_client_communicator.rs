use crate::communication::duplex_channel::DuplexChannel;
use ruft::{AddServerRequest, NewDataRequest, ClientRequestHandler};
use ruft::{ClientRpcResponse, ClientRequestChannels};
use std::time::Duration;
use crossbeam_channel::{Receiver, Sender};
use std::error::Error;

#[derive(Clone)]
pub struct InProcClientCommunicator {
	add_server_duplex_channel: DuplexChannel<AddServerRequest, ClientRpcResponse>,
	new_data_duplex_channel: DuplexChannel<NewDataRequest, ClientRpcResponse>
}

impl InProcClientCommunicator {
	pub fn new(node_id : u64, timeout : Duration) -> InProcClientCommunicator {
		InProcClientCommunicator {
			add_server_duplex_channel: DuplexChannel::new(format!("AddServer channel NodeId={}", node_id), timeout),
			new_data_duplex_channel: DuplexChannel::new(format!("NewData channel NodeId={}", node_id), timeout)
		}
	}


}

impl ClientRequestChannels for InProcClientCommunicator {
	fn add_server_request_rx(&self) -> Receiver<AddServerRequest> {
		self.add_server_duplex_channel.get_request_rx()
	}

	fn add_server_response_tx(&self) -> Sender<ClientRpcResponse> {
		self.add_server_duplex_channel.get_response_tx()
	}

	fn new_data_request_rx(&self) -> Receiver<NewDataRequest> {
		self.new_data_duplex_channel.get_request_rx()
	}

	fn new_data_response_tx(&self) -> Sender<ClientRpcResponse> {
		self.new_data_duplex_channel.get_response_tx()
	}
}

impl ClientRequestHandler for InProcClientCommunicator{
	fn add_server(&self, request: AddServerRequest) -> Result<ClientRpcResponse, Box<Error>> {
		trace!("Add server request {:?}", request);
		self.add_server_duplex_channel.send_request(request)
	}

	fn new_data(&self, request: NewDataRequest) -> Result<ClientRpcResponse, Box<Error>> {
		trace!("New data request {:?}", request);
		self.new_data_duplex_channel.send_request(request)
	}
}
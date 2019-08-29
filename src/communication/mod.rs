pub mod peers;
pub mod client;
pub mod duplex_channel;
pub mod peer_notifier;

pub trait QuorumResponse {
	fn get_result(&self) -> bool;
}
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use futures::future;
use tower_grpc::{Request, Response};

use raft::RaftError;
use raft::{
    DataEntryContent, EntryContent, LogEntry, NewClusterConfigurationEntryContent,
    PeerRequestChannels, PeerRequestHandler,
};

use crate::communication::duplex_channel::DuplexChannel;
use crate::communication::network::peer_communicator::client_requests::{
    append_entries_request, vote_request,
};
use crate::communication::network::peer_communicator::grpc::generated::grpc_peer_communicator::{
    server, AppendEntriesRequest, AppendEntriesResponse, VoteRequest, VoteResponse,
};
use crate::communication::network::peer_communicator::service_discovery::PeerCommunicatorServiceDiscovery;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct NetworkPeerCommunicator<Psd>
where
    Psd: PeerCommunicatorServiceDiscovery,
{
    node_id: u64,
    timeout: Duration,
    host: String,
    votes_channels: Arc<RwLock<HashMap<u64, DuplexChannel<raft::VoteRequest, raft::VoteResponse>>>>,
    append_entries_channels: Arc<
        RwLock<
            HashMap<u64, DuplexChannel<raft::AppendEntriesRequest, raft::AppendEntriesResponse>>,
        >,
    >,
    service_discovery: Psd,
}

impl<Psd> NetworkPeerCommunicator<Psd>
where
    Psd: PeerCommunicatorServiceDiscovery,
{
    pub fn new(
        host: String,
        node_id: u64,
        timeout: Duration,
        run_server: bool,
        service_discovery: Psd,
    ) -> NetworkPeerCommunicator<Psd> {
        let votes_channels = Arc::new(RwLock::new(HashMap::new()));
        let append_entries_channels = Arc::new(RwLock::new(HashMap::new()));

        let comm = NetworkPeerCommunicator {
            node_id,
            timeout,
            host,
            votes_channels,
            append_entries_channels,
            service_discovery,
        };

        if run_server {
            let comm_clone = comm.clone();
            let listen_address = comm_clone.get_address();
            thread::spawn(move || super::server::run_server(listen_address, comm_clone));
        }
        comm
    }

    fn add_node_communication(&self, node_id: u64) {
        let vote_duplex =
            DuplexChannel::new(format!("Vote channel NodeId={}", node_id), self.timeout);
        let append_entries_duplex = DuplexChannel::new(
            format!("AppendEntries channel NodeId={}", node_id),
            self.timeout,
        );

        self.votes_channels
            .write()
            .expect("acquire write lock")
            .insert(node_id, vote_duplex);

        self.append_entries_channels
            .write()
            .expect("acquire write lock")
            .insert(node_id, append_entries_duplex);
    }

    fn ensure_channels(&self, node_id: u64) {
        if !self
            .votes_channels
            .read()
            .expect("acquire read lock")
            .contains_key(&node_id)
        {
            self.add_node_communication(node_id);
        }
    }

    fn vote_request_tx(&self, node_id: u64) -> Sender<raft::VoteRequest> {
        self.ensure_channels(node_id);

        self.votes_channels.read().expect("acquire read lock")[&node_id].get_request_tx()
    }

    fn vote_response_rx(&self, node_id: u64) -> Receiver<raft::VoteResponse> {
        self.ensure_channels(node_id);

        self.votes_channels.read().expect("acquire read lock")[&node_id].get_response_rx()
    }

    fn append_entries_request_tx(&self, node_id: u64) -> Sender<raft::AppendEntriesRequest> {
        self.ensure_channels(node_id);

        self.append_entries_channels
            .read()
            .expect("acquire read lock")[&node_id]
            .get_request_tx()
    }

    fn append_entries_response_rx(&self, node_id: u64) -> Receiver<raft::AppendEntriesResponse> {
        self.ensure_channels(node_id);

        self.append_entries_channels
            .read()
            .expect("acquire read lock")[&node_id]
            .get_response_rx()
    }

    pub fn get_address(&self) -> SocketAddr {
        self.host.parse().unwrap()
    }
}

impl<Psd> PeerRequestChannels for NetworkPeerCommunicator<Psd>
where
    Psd: PeerCommunicatorServiceDiscovery,
{
    fn vote_request_rx(&self, node_id: u64) -> Receiver<raft::VoteRequest> {
        self.ensure_channels(node_id);

        self.votes_channels.read().expect("acquire read lock")[&node_id].get_request_rx()
    }

    fn vote_response_tx(&self, node_id: u64) -> Sender<raft::VoteResponse> {
        self.ensure_channels(node_id);

        self.votes_channels.read().expect("acquire read lock")[&node_id].get_response_tx()
    }

    fn append_entries_request_rx(&self, node_id: u64) -> Receiver<raft::AppendEntriesRequest> {
        self.ensure_channels(node_id);

        self.append_entries_channels
            .read()
            .expect("acquire read lock")[&node_id]
            .get_request_rx()
    }

    fn append_entries_response_tx(&self, node_id: u64) -> Sender<raft::AppendEntriesResponse> {
        self.ensure_channels(node_id);

        self.append_entries_channels
            .read()
            .expect("acquire read lock")[&node_id]
            .get_response_tx()
    }
}

impl<Psd> PeerRequestHandler for NetworkPeerCommunicator<Psd>
where
    Psd: PeerCommunicatorServiceDiscovery,
{
    fn send_vote_request(
        &self,
        destination_node_id: u64,
        request: raft::VoteRequest,
    ) -> Result<raft::VoteResponse, RaftError> {
        let host = self.service_discovery.get_address(destination_node_id);

        trace!("Destination Node {}. Address ({}). Vote request {:?}", destination_node_id, host, request);

        vote_request(host, self.timeout, request)
    }

    fn send_append_entries_request(
        &self,
        destination_node_id: u64,
        request: raft::AppendEntriesRequest,
    ) -> Result<raft::AppendEntriesResponse, RaftError> {
        let host = self.service_discovery.get_address(destination_node_id);

        trace!("Destination Node {}. Address ({}). Append entries request {}", destination_node_id, host, request);

        append_entries_request(host, self.timeout, request)
    }
}

impl<Psd> server::PeerRequestHandler for NetworkPeerCommunicator<Psd>
where
    Psd: PeerCommunicatorServiceDiscovery,
{
    type AppendEntriesFuture =
        future::FutureResult<Response<AppendEntriesResponse>, tower_grpc::Status>;
    type RequestVoteFuture = future::FutureResult<Response<VoteResponse>, tower_grpc::Status>;

    fn append_entries(
        &mut self,
        request: Request<AppendEntriesRequest>,
    ) -> Self::AppendEntriesFuture {
        trace!("Append entries request {:?}", request);

        let raft_request = convert_append_entries_request(request);

        let send_result = self
            .append_entries_request_tx(self.node_id)
            .send_timeout(raft_request, self.timeout);
        if let Err(err) = send_result {
            return future::err(tower_grpc::Status::new(
                tower_grpc::Code::Unknown,
                format!(" error:{}", err),
            ));
        }

        let receive_result = self
            .append_entries_response_rx(self.node_id)
            .recv_timeout(self.timeout);
        if let Err(err) = receive_result {
            return future::err(tower_grpc::Status::new(
                tower_grpc::Code::Unknown,
                format!(" error:{}", err),
            ));
        }

        if let Ok(resp) = receive_result {
            let response = Response::new(AppendEntriesResponse {
                term: resp.term,
                success: resp.success,
            });
            return future::ok(response);
        }

        unreachable!("invalid receive-response sequence");
    }

    fn request_vote(&mut self, request: Request<VoteRequest>) -> Self::RequestVoteFuture {
        trace!("Vote request {:?}", request);

        let raft_request = raft::VoteRequest {
            term: request.get_ref().term,
            candidate_id: request.get_ref().candidate_id,
            last_log_term: request.get_ref().last_log_term,
            last_log_index: request.get_ref().last_log_index,
        };

        let send_result = self
            .vote_request_tx(self.node_id)
            .send_timeout(raft_request, self.timeout);
        if let Err(err) = send_result {
            return future::err(tower_grpc::Status::new(
                tower_grpc::Code::Unknown,
                format!(" error:{}", err),
            ));
        }

        let receive_result = self
            .vote_response_rx(self.node_id)
            .recv_timeout(self.timeout);
        if let Err(err) = receive_result {
            return future::err(tower_grpc::Status::new(
                tower_grpc::Code::Unknown,
                format!(" error:{}", err),
            ));
        }

        if let Ok(resp) = receive_result {
            let response = Response::new(VoteResponse {
                term: resp.term,
                vote_granted: resp.vote_granted,
                peer_id: resp.peer_id,
            });
            return future::ok(response);
        }

        unreachable!("invalid receive-response sequence");
    }
}

fn convert_append_entries_request(
    request: Request<AppendEntriesRequest>,
) -> raft::AppendEntriesRequest {
    let req = request.into_inner();
    let entries = req
        .entries
        .into_iter()
        .map(|entry| {
            let content = match entry.content_type {
                1 => {
                    let inner_vec = entry.data;
                    let data = inner_vec.into_boxed_slice();

                    EntryContent::Data(DataEntryContent {
                        data: Arc::new(Box::leak(data)),
                    })
                }
                2 => EntryContent::AddServer(NewClusterConfigurationEntryContent {
                    new_cluster_configuration: entry.new_cluster_configuration.clone(),
                }),
                _ => panic!("invalid content type"),
            };

            LogEntry {
                term: entry.term,
                index: entry.index,
                entry_content: content,
            }
        })
        .collect();
    let raft_req = raft::AppendEntriesRequest {
        term: req.term,
        prev_log_term: req.prev_log_term,
        prev_log_index: req.prev_log_index,
        leader_id: req.leader_id,
        leader_commit: req.leader_commit,
        entries,
    };

    raft_req
}

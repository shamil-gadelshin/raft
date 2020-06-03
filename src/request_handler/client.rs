use crate::communication::client::{
    ClientRequestChannels, ClientResponseStatus, ClientRpcResponse,
};
use crate::communication::peers::PeerRequestHandler;
use crate::errors::RaftError;
use crate::node::state::{NodeStateSaver, NodeStatus, ProtectedNode};
use crate::operation_log::{
    DataEntryContent, EntryContent, NewClusterConfigurationEntryContent, OperationLog,
};
use crate::rsm::ReplicatedStateMachine;
use crate::{errors, Cluster};
use crossbeam_channel::Receiver;

pub struct ClientRequestHandlerParams<Log, Rsm, Cc, Pc, Ns, Cl>
where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Pc: PeerRequestHandler,
    Cc: ClientRequestChannels,
    Ns: NodeStateSaver,
    Cl: Cluster,
{
    pub protected_node: ProtectedNode<Log, Rsm, Pc, Ns, Cl>,
    pub client_communicator: Cc,
    pub cluster_configuration: Cl,
    pub max_data_content_size: u64,
}

pub fn process_client_requests<Log, Rsm, Cc, Pc, Ns, Cl>(
    params: ClientRequestHandlerParams<Log, Rsm, Cc, Pc, Ns, Cl>,
    terminate_worker_rx: Receiver<()>,
) where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Pc: PeerRequestHandler,
    Cc: ClientRequestChannels,
    Ns: NodeStateSaver,
    Cl: Cluster,
{
    info!("Client request processor worker started");
    let add_server_request_rx = params.client_communicator.add_server_request_rx();
    let new_data_request_rx = params.client_communicator.new_data_request_rx();
    loop {
        let response_tx;
        let entry_content;
        select!(
            recv(terminate_worker_rx) -> res  => {
                if res.is_err() {
                    error!("Abnormal exit for client request processor worker");
                }
                break
            },
            recv(add_server_request_rx) -> res => {
                let request = res.expect("can get add server request");
                entry_content = EntryContent::AddServer(NewClusterConfigurationEntryContent {
                     new_cluster_configuration: get_new_configuration(
                         params.cluster_configuration.clone(),
                         request.new_server)
                 });
                response_tx = params.client_communicator.add_server_response_tx();

           },
            recv(new_data_request_rx) -> res => {
                let request = res.expect("can get new_data request");
                entry_content = EntryContent::Data(DataEntryContent { data: request.data });
                response_tx = params.client_communicator.new_data_response_tx();
            },
        );

        let client_rpc_response_result = process_client_request_internal(
            params.protected_node.clone(),
            entry_content,
            params.max_data_content_size,
        );

        let process_request_result = match client_rpc_response_result {
            Ok(client_rpc_response) => {
                let send_result = response_tx.send(client_rpc_response);
                if let Err(err) = send_result {
                    errors::new_err("cannot send clientRpcResponse".to_string(), err.to_string())
                } else {
                    Ok(())
                }
            }
            Err(err) => {
                let msg = format!("Process client request error: {}", err);
                error!("{}", msg);
                errors::new_err("Cannot send clientRpcResponse".to_string(), msg)
            }
        };

        trace!("Client request processed: {:?}", process_request_result)
    }
    info!("Client request processor worker stopped");
}

fn process_client_request_internal<Log, Rsm, Pc, Ns, Cl>(
    protected_node: ProtectedNode<Log, Rsm, Pc, Ns, Cl>,
    entry_content: EntryContent,
    max_data_content_size: u64,
) -> Result<ClientRpcResponse, RaftError>
where
    Log: OperationLog,
    Rsm: ReplicatedStateMachine,
    Pc: PeerRequestHandler,
    Ns: NodeStateSaver,
    Cl: Cluster,
{
    let mut node = protected_node.lock();

    if let EntryContent::Data(content) = &entry_content {
        if content.data.len() as u64 > max_data_content_size {
            return Ok(ClientRpcResponse {
                status: ClientResponseStatus::Error,
                current_leader: node.current_leader_id,
                message: format!(
                    "Max data size ({}) exceeded:{}",
                    max_data_content_size,
                    content.data.len()
                ),
            });
        }
    }

    let client_rpc_response = match node.status {
        NodeStatus::Leader => {
            let append_result = node.append_content_to_log(entry_content);

            match append_result {
                Err(err) => ClientRpcResponse {
                    status: ClientResponseStatus::Error,
                    current_leader: node.current_leader_id,
                    message: format!("{}", err),
                },
                Ok(quorum_gathered) => {
                    if !quorum_gathered {
                        ClientRpcResponse {
                            status: ClientResponseStatus::NoQuorum,
                            current_leader: node.current_leader_id,
                            message: String::new(),
                        }
                    } else {
                        ClientRpcResponse {
                            status: ClientResponseStatus::Ok,
                            current_leader: node.current_leader_id,
                            message: String::new(),
                        }
                    }
                }
            }
        }
        NodeStatus::Candidate | NodeStatus::Follower => ClientRpcResponse {
            status: ClientResponseStatus::NotLeader,
            current_leader: node.current_leader_id,
            message: String::new(),
        },
    };

    Ok(client_rpc_response)
}

fn get_new_configuration<Cl>(cluster_configuration: Cl, new_server: u64) -> Vec<u64>
where
    Cl: Cluster,
{
    let mut nodes = cluster_configuration.all_nodes();
    nodes.push(new_server);

    nodes
}

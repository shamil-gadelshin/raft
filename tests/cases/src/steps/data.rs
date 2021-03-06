use crate::steps::cluster::Leader;
use raft::{ClientRequestHandler, ClientRpcResponse, NewDataRequest, RaftError};
use std::sync::Arc;

pub fn add_data_sample<Cc: ClientRequestHandler>(
    leader: &Leader<Cc>,
) -> Result<ClientRpcResponse, RaftError> {
    let bytes = b"small size data sample";
    let new_data_request = NewDataRequest {
        data: Arc::new(bytes),
    };
    let data_resp = leader.client_handler.new_data(new_data_request);
    info!(
        "Add data request sent for Node {}. Response = {:?}",
        leader.id, data_resp
    );

    data_resp
}

pub fn add_tiny_data_sample<Cc: ClientRequestHandler>(
    leader: &Leader<Cc>,
) -> Result<ClientRpcResponse, RaftError> {
    let bytes = b"tiny";
    let new_data_request = NewDataRequest {
        data: Arc::new(bytes),
    };
    let data_resp = leader.client_handler.new_data(new_data_request);
    info!(
        "Add data request sent for Node {}. Response = {:?}",
        leader.id, data_resp
    );

    data_resp
}

pub fn add_server<Cc: ClientRequestHandler>(leader: &Leader<Cc>, new_node_id: u64) {
    let add_server_request = raft::AddServerRequest {
        new_server: new_node_id,
    };
    let resp = leader.client_handler.add_server(add_server_request);
    info!(
        "Add server request sent for Node {}. Response = {:?}",
        leader.id, resp
    );
}

pub fn add_ten_thousands_data_samples<Cc: ClientRequestHandler>(leader: Leader<Cc>) {
    fn add_thousands_of_data<Cc: ClientRequestHandler + ?Sized + Sync>(client_handler: Arc<Cc>) {
        let bytes = b"lot of small data";
        let data_request = NewDataRequest {
            data: Arc::new(bytes),
        };
        for _count in 1..=10000 {
            let _resp = client_handler.new_data(data_request.clone());
        }
    }

    add_thousands_of_data(leader.client_handler);
}

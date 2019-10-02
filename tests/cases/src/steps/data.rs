use crate::steps::cluster::Leader;
use raft::{ClientRequestHandler, ClientRpcResponse, NewDataRequest, RaftError};
use std::sync::Arc;

pub fn add_data_sample<Cc: ClientRequestHandler>(
    leader: &Leader<Cc>,
) -> Result<ClientRpcResponse, RaftError> {
    let bytes = "small size data sample".as_bytes();
    let new_data_request = NewDataRequest {
        data: Arc::new(bytes),
    };
    let data_resp = leader.client_handler.new_data(new_data_request.clone());
    info!("Add data request sent for Node {}. Response = {:?}", leader.id, data_resp);

    data_resp
}

pub fn add_tiny_data_sample<Cc: ClientRequestHandler>(
    leader: &Leader<Cc>,
) -> Result<ClientRpcResponse, RaftError> {
    let bytes = "tiny".as_bytes();
    let new_data_request = NewDataRequest {
        data: Arc::new(bytes),
    };
    let data_resp = leader.client_handler.new_data(new_data_request.clone());
    info!("Add data request sent for Node {}. Response = {:?}", leader.id, data_resp);

    data_resp
}

pub fn add_server<Cc: ClientRequestHandler>(leader: &Leader<Cc>, new_node_id: u64) {
    let add_server_request = raft::AddServerRequest {
        new_server: new_node_id,
    };
    let resp = leader.client_handler.add_server(add_server_request);
    info!("Add server request sent for Node {}. Response = {:?}", leader.id, resp);
}

pub fn add_ten_thousands_data_samples<Cc: ClientRequestHandler>(leader: Leader<Cc>) {
    fn add_thousands_of_data<Cc: ClientRequestHandler + ?Sized + Sync>(client_handler: Arc<Cc>) {
        //  thread::sleep(Duration::from_secs(7));
        let bytes = "lot of small data".as_bytes();
        let data_request = NewDataRequest {
            data: Arc::new(bytes),
        };
        for _count in 1..=10000 {
            let _resp = client_handler.new_data(data_request.clone());
        }
    }

    add_thousands_of_data(leader.client_handler);
    //	thread::spawn(     ||add_thousands_of_data(leader.client_handler));
}

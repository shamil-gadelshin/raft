extern crate log;
extern crate env_logger;
extern crate chrono;
extern crate crossbeam_channel;

use std::collections::HashMap;
use std::io::Write;

use chrono::prelude::{DateTime, Local};

use std::sync::Arc;
use std::time::Duration;
use raft_modules::NetworkClientCommunicator;
use raft::{ClientRequestHandler, NewDataRequest, ClientResponseStatus};

fn main() {
    init_logger();

	let node_id = 1;
	let communication_timeout = Duration::from_millis(1000);

    let client_request_handler = NetworkClientCommunicator::new(
        get_address(node_id),
        node_id,
        communication_timeout,
        false);

    send_data(client_request_handler);
}

fn send_data<Cc : ClientRequestHandler>(client_request_handler: Cc) {
    let bytes = "just data".as_bytes();
    let new_data_request = NewDataRequest { data: Arc::new(bytes) };
    let result = client_request_handler.new_data(new_data_request);

    println!("{:?}", result);
//    if let Ok(resp) = result {
//        println!("{:?}", resp);
//        if let ClientResponseStatus::Ok = resp.status {
//
//        }
//    }
}



fn init_logger() {
    env_logger::builder()
        .format(|buf, record| {
            let now: DateTime<Local> = Local::now();
            writeln!(buf, "{:5}: {} - {}", record.level(), now.format("%H:%M:%S.%3f").to_string(), record.args())
        })
        .init();
}



pub fn get_address(node_id : u64) -> String{
    format!("127.0.0.1:{}", 50000 + node_id)
}


#[allow(dead_code)]
fn find_a_leader<Cc : ClientRequestHandler>(client_handlers : HashMap<u64, Cc>) -> u64{
    let bytes = "find a leader".as_bytes();
    let new_data_request = NewDataRequest{data : Arc::new(bytes)};
    for kv in client_handlers {
        let (_k, v) = kv;

        let result = v.new_data(new_data_request.clone());
        if let Ok(resp) = result {
            if let ClientResponseStatus::Ok = resp.status {
                return resp.current_leader.expect("can get a leader");
            }
        }
    }

    panic!("cannot get a leader!")
}

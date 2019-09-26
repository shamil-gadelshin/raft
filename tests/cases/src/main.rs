#[macro_use] extern crate log;
extern crate env_logger;
extern crate chrono;
extern crate crossbeam_channel;

mod steps;
mod cases;

use std::io::Write;
use chrono::prelude::{DateTime, Local};

extern crate raft;
extern crate raft_modules;



fn init_logger() {
    env_logger::builder()
        .format(|buf, record| {
            let now: DateTime<Local> = Local::now();
            writeln!(buf, "{:5}: {} - {}", record.level(), now.format("%H:%M:%S.%3f").to_string(), record.args())
        })
        .init();
}


fn main() {
    init_logger();

    cases::smoke::run();
    cases::add_thousand::run();
    cases::no_quorum::run();
    cases::single_node::run();
    cases::operation_log::basic_replication::run()
}








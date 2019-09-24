#[macro_use] extern crate log;
extern crate env_logger;
extern crate chrono;

use std::io::Write;
use chrono::prelude::{DateTime, Local};


extern crate cases;

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

    info!("Stand-alone Smoke test started");

    cases::smoke::run();

    info!("Stand-alone Smoke test completed");
}

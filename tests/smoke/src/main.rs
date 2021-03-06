#[macro_use]
extern crate log;
extern crate chrono;
extern crate env_logger;

use chrono::prelude::{DateTime, Local};
use std::io::Write;

extern crate cases;

fn init_logger() {
    env_logger::builder()
        .format(|buf, record| {
            let now: DateTime<Local> = Local::now();
            let now_str = now.format("%H:%M:%S.%3f").to_string();
            writeln!(buf, "{:5}: {} - {}", record.level(), now_str, record.args())
        })
        .init();
}

fn main() {
    init_logger();

    info!("Stand-alone Smoke test started");

    cases::smoke::run();

    info!("Stand-alone Smoke test completed");
}

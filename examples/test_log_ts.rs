use log::LevelFilter;
use log_derive::logfn;
use simplelog::{Config, TermLogger, TerminalMode};
use std::thread::sleep;
use std::time::Duration;

#[logfn(DEBUG)]
fn func1() ->i32 {
    sleep(Duration::from_millis(10));
    log::info!("func1");
    5
}

#[logfn(DEBUG, log_ts = true)]
fn func2() {
    sleep(Duration::from_millis(1500));
    log::info!("func2");
}

#[logfn(DEBUG, log_ts = false)]
fn func3() {
    log::info!("func3");
}

fn main() {
    TermLogger::init(LevelFilter::Trace, Config::default(), TerminalMode::default()).unwrap();
    func1();
    func2();
    func3();
}

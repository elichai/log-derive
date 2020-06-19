use log::LevelFilter;
use log_derive::logfn;
use simplelog::{Config, TermLogger, TerminalMode};
use std::thread::sleep;
use std::time::Duration;

#[logfn(DEBUG)]
async fn func1() -> i32 {
    sleep(Duration::from_millis(10));
    log::info!("func1");
    5
}

#[logfn(DEBUG, log_ts = true)]
async fn func2() {
    sleep(Duration::from_millis(200));
    log::info!("func2");
}

#[logfn(DEBUG, log_ts = false)]
async fn func3() {
    log::info!("func3");
}

fn main() {
    TermLogger::init(LevelFilter::Trace, Config::default(), TerminalMode::default()).unwrap();
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(func1());
    rt.block_on(func2());
    rt.block_on(func3());
}

#[cfg(test)]
#[test]
fn test_main() {
    main();
}

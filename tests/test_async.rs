#![cfg(feature = "async_test")]

use log_derive::logfn;

#[logfn(INFO)]
async fn async_function(ok: bool) -> Result<&'static str, &'static str> {
    if ok {
        return Ok("async Ok");
    } else {
        return Err("async Err");
    }
}

#[test]
fn async_works() {
    init_logger();

    futures_executor::block_on(async {
        assert_eq!(async_function(true).await, Ok("async Ok"));
        assert_eq!(async_function(false).await, Err("async Err"));
    })
}

fn init_logger() {
    // Run with `cargo test -- --test-threads=1 --nocapture`
    let _ = simplelog::SimpleLogger::init(simplelog::LevelFilter::Info, simplelog::Config::default());
}

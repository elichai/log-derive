#![cfg(feature = "async_test")]

mod test_logger;

use crate::test_logger::THREAD_LOGGER;
use log::Level;
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
    test_logger::init();

    futures_executor::block_on(async {
        assert_eq!(async_function(true).await, Ok("async Ok"));
        THREAD_LOGGER.assert_last_log("async_function() => \"async Ok\"", Level::Info, 9);
        assert_eq!(async_function(false).await, Err("async Err"));
        THREAD_LOGGER.assert_last_log("async_function() => \"async Err\"", Level::Info, 9);
        assert!(THREAD_LOGGER.is_empty())
    })
}

#[test]
fn async_works2() {
    test_logger::init();
    let block = futures_executor::block_on;

    assert_eq!(block(async_function(true)), Ok("async Ok"));
    THREAD_LOGGER.assert_last_log("async_function() => \"async Ok\"", Level::Info, 9);
    assert_eq!(block(async_function(false)), Err("async Err"));
    THREAD_LOGGER.assert_last_log("async_function() => \"async Err\"", Level::Info, 9);
    assert!(THREAD_LOGGER.is_empty())
}

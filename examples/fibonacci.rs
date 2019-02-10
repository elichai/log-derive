use log_derive::logfn;
use simplelog::{TermLogger, Config};
use log::LevelFilter;

// #[logfn(INFO, fmt = "fibonacci() -> {:?}", err = "Error", ok = "Trace", Warn)]
// fn fibonacci(n: u32) -> std::result::Result<u32, u32> {
//     match n {
//         0 => Ok(1),
//         1 => Ok(1),
//         3 => Err(3),
//         _ => Ok(fibonacci(n - 1)? + fibonacci(n - 2)?),
//     }
// }

#[logfn(INFO, fmt = "fibonacci() -> {}", ok = "Trace")]
fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 1,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}


fn main() {
    TermLogger::init(LevelFilter::Trace, Config::default()).unwrap();
    let _ = fibonacci(5);
}

#[cfg(test)]
#[test]
fn test_main() {
    main();
}
use std::fmt;

use log_derive::logfn;
use simplelog::{TermLogger, Config};
use log::LevelFilter;

struct DivisibleBy3Error(u32);

impl fmt::Display for DivisibleBy3Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} is divisible by 3", self.0)
    }
}

#[logfn(fmt = "not_divisible_by_3() -> {}", ok = "info", err = "error")]
fn not_divisible_by_3(n: u32) -> Result<u32, DivisibleBy3Error> {
    match n % 3 {
        0 => Err(DivisibleBy3Error(n)),
        _ => Ok(n)
    }
}


fn main() {
    TermLogger::init(LevelFilter::Trace, Config::default()).unwrap();
    for x in 0..25 {
        let _ = not_divisible_by_3(x);
    }
}
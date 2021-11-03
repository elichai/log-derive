use std::fmt;

use log::LevelFilter;
use log_derive::logfn;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};

struct DivisibleBy3Error(u32);
struct DivisibleBy5Error(u32);
struct DivisibleBy7Error(u32);

type MyResult<T> = Result<u32, T>;

impl fmt::Display for DivisibleBy3Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} is divisible by 3", self.0)
    }
}
impl fmt::Display for DivisibleBy7Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} is divisible by 7", self.0)
    }
}

impl fmt::Debug for DivisibleBy5Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} is divisible by 5", self.0)
    }
}

#[logfn(fmt = "not_divisible_by_3() -> {}", ok = "info", err = "error")]
fn not_divisible_by_3(n: u32) -> Result<u32, DivisibleBy3Error> {
    match n % 3 {
        0 => Err(DivisibleBy3Error(n)),
        _ => Ok(n),
    }
}

#[logfn(Info, fmt = "not_divisible_by_5() -> {:?}")]
fn not_divisible_by_5_with_enum_wrap(n: u32) -> MyResult<DivisibleBy5Error> {
    match n % 5 {
        0 => Err(DivisibleBy5Error(n)),
        _ => Ok(n),
    }
}

#[logfn(fmt = "not_divisible_by_7() -> {}", ok = "info", err = "error")]
fn not_divisible_by_7(n: u32) -> MyResult<DivisibleBy7Error> {
    match n % 7 {
        0 => Err(DivisibleBy7Error(n)),
        _ => Ok(n),
    }
}

fn main() {
    TermLogger::init(LevelFilter::Trace, Config::default(), TerminalMode::default(), ColorChoice::Auto).unwrap();
    for x in 0..25 {
        let _ = not_divisible_by_3(x);
        let _ = not_divisible_by_5_with_enum_wrap(x);
        let _ = not_divisible_by_7(x);
    }
}

#[cfg(test)]
#[test]
fn test_main() {
    main();
}

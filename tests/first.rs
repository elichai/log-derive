mod test_logger;

use crate::test_logger::THREAD_LOGGER;
use log::Level;
use log_derive::{logfn, logfn_inputs};

#[logfn(INFO, fmt = "wrapper_function returned {:?}")]
fn wrapped_function(a: u8, b: &str) {
    let mut test1 = Vec::new();
    let mut test2 = || {
        test1.push(5);
    };
    test2();
    println!("{} {}", b, a);
}

struct AAAAAA;
impl AAAAAA {
    #[logfn(Info)]
    pub fn yoyoy(&self, _a: String, _b: u8, _c: Vec<u8>) -> Vec<u8> {
        vec![0u8; 8]
    }
}

#[derive(Debug, PartialEq)]
struct E;

trait Test {
    fn abc(&mut self, err: Tes) -> Result<String, E>;
    fn third(&self, err: &Tes) -> Result<String, E>;
    fn just_inputs(&self, err: &Tes) -> Result<String, E>;
    fn both(&self, err: &Tes) -> Result<String, E>;
}

#[derive(Debug)]
struct Me(Option<u8>);
#[derive(Debug)]
struct Tes(pub bool);

impl Test for Me {
    #[logfn(INFO, fmt = "DB: {:?}", ok = "debug", err = "trace")]
    fn abc(&mut self, err: Tes) -> Result<String, E> {
        let mut clos = || {
            self.third(&err)?;
            if err.0 {
                return Err(E);
            } else {
                self.0 = Some(5);
                return Ok(String::from("Hi!"));
            }
        };
        let result = clos();
        result
    }

    #[logfn(Info)]
    fn third(&self, err: &Tes) -> Result<String, E> {
        if err.0 {
            return Err(E);
        } else {
            return Ok(String::from("Hi!"));
        }
    }

    #[logfn_inputs(Debug)]
    fn just_inputs(&self, err: &Tes) -> Result<String, E> {
        if err.0 {
            return Err(E);
        } else {
            return Ok(String::from("Hi!"));
        }
    }

    #[logfn_inputs(Trace)]
    #[logfn(Info)]
    fn both(&self, err: &Tes) -> Result<String, E> {
        let clos = || {
            self.third(&err)?;
            if err.0 {
                return Err(E);
            } else {
                return Ok(String::from("Hi!"));
            }
        };
        let result = clos();
        result
    }
}

#[test]
fn works() {
    test_logger::init();
    wrapped_function(5, "cool!");
    THREAD_LOGGER.assert_last_log("wrapper_function returned ()", Level::Info, 7);
    let a = AAAAAA;
    let _ = a.yoyoy(String::from("fds"), 55, vec![1u8; 12]);
    THREAD_LOGGER.assert_last_log("yoyoy() => [0, 0, 0, 0, 0, 0, 0, 0]", Level::Info, 19);
    let mut b = Me(None);
    let tes = Tes(false);
    b.abc(tes).unwrap();
    THREAD_LOGGER.assert_last_log("DB: \"Hi!\"", Level::Debug, 41);
    // `b.abc` calls `third()` so we need to assert that log too.
    THREAD_LOGGER.assert_last_log("third() => \"Hi!\"", Level::Info, 56);
    assert!(THREAD_LOGGER.is_empty())
}

#[test]
fn test_inputs() {
    test_logger::init();
    let b = Me(Some(5));
    let tes = Tes(false);
    b.just_inputs(&tes).unwrap();
    THREAD_LOGGER.assert_last_log("just_inputs(self: Me(Some(5)),err: Tes(false))", Level::Debug, 65);
    b.both(&tes).unwrap();

    // Assert `b.both` input log
    THREAD_LOGGER.assert_last_log("both() => \"Hi!\"", Level::Info, 75);
    // `b.both` calls `third()` so wee need to assert that log too
    THREAD_LOGGER.assert_last_log("third() => \"Hi!\"", Level::Info, 56);
    // Assert `b.both` output log
    // Due to a bug in rust stable we can't test the line number here. (rust-lang/rust#74035)
    // THREAD_LOGGER.assert_last_log("both(self: Me(Some(5)),err: Tes(false))", Level::Trace, 74);
    let log = THREAD_LOGGER.pop_log();
    assert_eq!(log.msg, "both(self: Me(Some(5)),err: Tes(false))");
    assert_eq!(log.level, Level::Trace);
    assert!(THREAD_LOGGER.is_empty())
}

#[test]
fn fail() {
    test_logger::init();
    wrapped_function(5, "cool!");
    THREAD_LOGGER.assert_last_log("wrapper_function returned ()", Level::Info, 7);
    let a = AAAAAA;
    let _ = a.yoyoy(String::from("fds"), 55, vec![1u8; 12]);
    THREAD_LOGGER.assert_last_log("yoyoy() => [0, 0, 0, 0, 0, 0, 0, 0]", Level::Info, 19);

    let mut b = Me(None);
    let tes = Tes(true);
    assert_eq!(b.abc(tes), Err(E));
    THREAD_LOGGER.assert_last_log("DB: E", Level::Trace, 41);
    // `b.abc` calls `third()` so wee need to assert that log too
    THREAD_LOGGER.assert_last_log("third() => E", Level::Info, 56);

    assert!(THREAD_LOGGER.is_empty())
}

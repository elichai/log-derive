use log::{Level, LevelFilter, Log, Metadata, Record};
use std::cell::RefCell;

#[derive(Clone, Debug)]
pub struct LogRecord {
    msg: String,
    level: Level,
    line: u32,
}

struct VecLooger(Vec<LogRecord>);

// Should assert this in any test because TLS doesn't always promise destructors are executed.
// This is implemented as a sanity so we never forget to assert we emptied the Logger.
impl Drop for VecLooger {
    fn drop(&mut self) {
        assert!(self.0.is_empty());
    }
}

// The basic idea goes as follow:
// We use a public ZST static logger, but in the `Log` impl it actually calls into the thread_local,
// that way we don't need to worry about race conditions in the code because each test has it's own VecLogger.
pub struct ThreadSingletonLogger;
pub static THREAD_LOGGER: ThreadSingletonLogger = ThreadSingletonLogger;
thread_local! {static LOGGER: RefCell<VecLooger> = RefCell::new(VecLooger(Vec::with_capacity(4)));}

impl Log for ThreadSingletonLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        LOGGER.with(|cell| {
            let vec = &mut cell.borrow_mut().0;
            let new_log = LogRecord { msg: record.args().to_string(), level: record.level(), line: record.line().unwrap() };
            vec.push(new_log);
        })
    }

    fn flush(&self) {}
}

impl ThreadSingletonLogger {
    pub fn assert_last_log(&self, msg: &str, level: Level, line: u32) {
        LOGGER.with(|cell| {
            let last = cell.borrow_mut().0.pop().unwrap();
            assert_eq!(last.msg, msg);
            assert_eq!(last.level, level);
            assert_eq!(last.line, line);
        })
    }

    pub fn is_empty(&self) -> bool {
        LOGGER.with(|cell| cell.borrow().0.is_empty())
    }
}

pub fn init() {
    // This is thread safe because `set_logger` is atomic.
    let _ = log::set_logger(&THREAD_LOGGER);
    log::set_max_level(LevelFilter::max())
}

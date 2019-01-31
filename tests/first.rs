use log_derive::log;
use log::log;
#[log(INFO)]
fn wrapped_function(a: u8, b: &str) {
    println!("{} {}", b, a);
}

struct AAAAAA;
impl AAAAAA {
    #[log(Info)]
    pub fn yoyoy(&self, a: String, b: u8, c: Vec<u8>) -> Vec<u8> {
        log!(log::Level::Info, "Hi!");
        vec![0u8; 8]
    }
}

#[test]
fn works() {
    wrapped_function(5, "cool!");
}
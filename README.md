# log-derive
[![Build Status](https://travis-ci.com/elichai/log-derive.svg?branch=master)](https://travis-ci.com/elichai/log-derive)
[![Latest version](https://img.shields.io/crates/v/log-derive.svg)](https://crates.io/crates/log-derive)
[![Documentation](https://docs.rs/log-derive/badge.svg)](https://docs.rs/log-derive)
![License](https://img.shields.io/crates/l/log-derive.svg)

A Rust macro to part of the [log](https://crates.io/crates/log) facade that auto generates loggings for functions output. 

* [Documentation](https://docs.rs/log-derive)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
log-derive = "0.2"
log = "0.4"

```

and for Rust Edition 2015 add this to your crate root:

```rust
#[macro_use]
extern crate log_derive;
extern crate log;
```
In Rust Edition 2018 you can simply do:
```rust
use log_derive::logfn;
```

After that all you need is to add the macro above a function that returns an output that implements the `Debug` trait.

# Examples

```rust
 #[logfn(Err = "Error", fmt = "Failed Sending Packet: {:?}")]
 fn send_hi(addr: SocketAddr) -> Result<(), io::Error> {
     let mut stream = TcpStream::connect(addr)?;
     stream.write(b"Hi!")?;
     Ok( () )
 }

```

```rust
#[logfn(Trace)]
fn test_log(a: u8) -> String {
  (a*2).to_string()
}

```

# Output
The output of the [fibonacci](./examples/fibonacci.rs) example:
```
14:55:41 [INFO] fibonacci() -> 1
14:55:41 [INFO] fibonacci() -> 1
14:55:41 [INFO] fibonacci() -> 2
14:55:41 [INFO] fibonacci() -> 1
14:55:41 [INFO] fibonacci() -> 3
14:55:41 [INFO] fibonacci() -> 1
14:55:41 [INFO] fibonacci() -> 1
14:55:41 [INFO] fibonacci() -> 2
14:55:41 [INFO] fibonacci() -> 5
14:55:41 [INFO] fibonacci() -> 1
14:55:41 [INFO] fibonacci() -> 1
14:55:41 [INFO] fibonacci() -> 2
14:55:41 [INFO] fibonacci() -> 1
14:55:41 [INFO] fibonacci() -> 3
14:55:41 [INFO] fibonacci() -> 8
```

If you expand the output of the macro the resulting code will look something like this:
```rust
fn fibonacci(n: u32) -> u32 {
    let mut closure = || match n {
        0 => 1,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    };
    let result = closure();
    log::log!(log::Level::Info, "fibonacci() -> {}", result);
    result
}
```
Of course the `log!` macro will be expanded too and it will be a bit more messy.

If the function returns a `Result` it will match through it to split between the `Ok` LogLevel and the `Err` LogLevel
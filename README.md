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
```

and for Rust Edition 2015 add this to your crate root:

```rust
#[macro_use]
extern crate log_derive;
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

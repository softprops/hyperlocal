# hyperlocal

> [hyper](https://github.com/hyperium/hyper) bindings for [unix domain sockets](https://github.com/rust-lang-nursery/unix-socket)

## usage

```rust
extern crate hyper;
extern crate hyperlocal;

use hyperlocal::{SocketConnector, DomainUrl}
use hyper::Client;
use std::io;

fn main() {
  let client = Client::with_connector(SocketConnector);
  let mut res = client.get(
    DomainUrl("/path/to/socket", "/path/to/resource")
  );
  io::copy(&mut res, &mut io::stdout()).unwrap();
}
```

Doug Tangren (softprops) 2015

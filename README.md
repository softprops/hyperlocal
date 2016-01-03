# hyperlocal

[![Build Status](https://travis-ci.org/softprops/hyperlocal.svg?branch=master)](https://travis-ci.org/softprops/hyperlocal)

> [hyper](https://github.com/hyperium/hyper) client and server bindings for [unix domain sockets](https://github.com/rust-lang-nursery/unix-socket)

Hyper is a rock solid [rustlang](https://www.rust-lang.org/) HTTP client and server tool kit. [Unix domain sockets](https://en.wikipedia.org/wiki/Unix_domain_socket) provide
a mechanism for host-local interprocess communication. Hyperlocal builds on and complements hyper's interfaces for building unix domain socket HTTP clients and servers.

## api docs

Find them [here](https://softprops.github.com/hyperlocal)

## install

tbd

## usage

### servers

```rust
extern crate hyper;
extern crate hyperlocal;

use hyper::server::{Request, Response};
use hyperlocal::UnixSocketServer;

fn main() {
    let path = "test.sock";
    let server = UnixSocketServer::new(path).unwrap();
    server.handle(|_: Request, res: Response| {
        let _ = res.send(b"It's a Unix system. I know this.");
    }).unwrap();
    println!("listening @ {}", path);
}
```

### client

```rust
extern crate hyper;
extern crate hyperlocal;

use hyper::Client;
use hyperlocal::{DomainUrl, UnixSocketConnector};

fn main() {
    let path = "test.sock";
    let client = Client::with_connector(UnixSocketConnector);
    let mut res = client.get(DomainUrl::new(path, "/")).send().unwrap();
    std::io::copy(&mut res, &mut std::io::stdout()).unwrap();
}
```

Doug Tangren (softprops) 2015

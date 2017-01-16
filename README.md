# hyperlocal

[![Build Status](https://travis-ci.org/softprops/hyperlocal.svg?branch=master)](https://travis-ci.org/softprops/hyperlocal) [![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![crates.io](http://meritbadge.herokuapp.com/hyperlocal)](https://crates.io/crates/hyperlocal)

> [hyper](https://github.com/hyperium/hyper) client and server bindings for [unix domain sockets](https://github.com/rust-lang-nursery/unix-socket)

Hyper is a rock solid [rustlang](https://www.rust-lang.org/) HTTP client and server tool kit. [Unix domain sockets](https://en.wikipedia.org/wiki/Unix_domain_socket) provide
a mechanism for host-local interprocess communication. Hyperlocal builds on and complements hyper's interfaces for building unix domain socket HTTP clients and servers.

This is useful for exposing simple HTTP interfaces for your Unix daemons in cases where you want to limit access to the current host, in which case, opening and exposing tcp ports is not needed. Examples of unix daemons that provide this kind of host local interface include, [docker](https://docs.docker.com/engine/misc/), a process container manager.

## api docs

Find them [here](https://softprops.github.com/hyperlocal)

## install

Add the following to your `Cargo.toml` file

```toml
[dependencies]
hyperlocal = "0.3.0"
```

## usage

### servers

A typical server can be build with `hyperlocal::UnixSocketServer`

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

### clients

You can communicate over HTTP with Unix domain socket servers using hyper's Client interface.
Configure your hyper client using `Client::with_connector(UnixSocketConnector)`.

Hyper's client
interface makes it easy to issue typical HTTP methods like GET, POST, DELETE with factory methods,
`get`, `post`, `delete`, ect. These require an argument that can be tranformed into a `Url`.
Since unix domain sockets aren't represented hostnames that resolve to ip addresses coupled with network ports ports,
you standard url string won't do. Instead, use `DomainUrl`
which represents both file path to the domain socket and the resource uri path and query string.

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

Doug Tangren (softprops) 2015-2016

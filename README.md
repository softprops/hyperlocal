# hyperlocal

[![Build Status](https://travis-ci.org/softprops/hyperlocal.svg?branch=master)](https://travis-ci.org/softprops/hyperlocal) [![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![crates.io](http://meritbadge.herokuapp.com/hyperlocal)](https://crates.io/crates/hyperlocal)

> [hyper](https://github.com/hyperium/hyper) client and server bindings for [unix domain sockets](https://github.com/tokio-rs/tokio-uds)

Hyper is a rock solid [rustlang](https://www.rust-lang.org/) HTTP client and server tool kit. [Unix domain sockets](https://en.wikipedia.org/wiki/Unix_domain_socket) provide
a mechanism for host-local interprocess communication. Hyperlocal builds on and complements hyper's interfaces for building unix domain socket HTTP clients and servers.

This is useful for exposing simple HTTP interfaces for your Unix daemons in cases where you want to limit access to the current host, in which case, opening and exposing tcp ports is not needed. Examples of unix daemons that provide this kind of host local interface include, [docker](https://docs.docker.com/engine/misc/), a process container manager.

## [Documentation](https://softprops.github.com/hyperlocal)

## install

Add the following to your `Cargo.toml` file

```toml
[dependencies]
hyperlocal = "0.4"
```

## usage

### servers

A typical server can be built with `hyperlocal::server::Http`

note the example below uses a crate called `service_fn` which exists [here](https://github.com/tokio-rs/service-fn) but is not yet published to crates.io

```rust
extern crate hyper;
extern crate hyperlocal;
extern crate futures;
extern crate tokio_service;
extern crate service_fn;

use hyper::{Result, Response};
use hyper::header::{ContentType, ContentLength};
use service_fn::service_fn;

const PHRASE: &'static str = "It's a Unix system. I know this.";

fn run() -> Result<()> {
    let path = "test.sock";
    let hello = || {
        Ok(service_fn(|_| {
            Ok(
                Response::<hyper::Body>::new()
                    .with_header(ContentLength(PHRASE.len() as u64))
                    .with_header(ContentType::plaintext())
                    .with_body(PHRASE),
            )
        }))
    };
    let svr = hyperlocal::server::Http::new().bind(path, hello)?;
    println!("Listening on unix://{path} with 1 thread.", path = path);
    svr.run()?;
    Ok(())
}

fn main() {
    run().unwrap()
}
```

### clients

You can communicate over HTTP with Unix domain socket servers using hyper's Client interface.
Configure your hyper client using `Client::configure(...)`.

Hyper's client
interface makes it easy to issue typical HTTP methods like GET, POST, DELETE with factory methods,
`get`, `post`, `delete`, ect. These require an argument that can be tranformed into a `hyper::Uri`.
Since unix domain sockets aren't represented with hostnames that resolve to ip addresses coupled with network ports ports,
your standard url string won't do. Instead, use a `hyperlocal::Uri`
which represents both file path to the domain socket and the resource uri path and query string.

```rust
extern crate futures;
extern crate hyper;
extern crate hyperlocal;
extern crate tokio_core;

use std::io::{self, Write};

use futures::Stream;
use futures::Future;
use hyper::{Client, Result};
use hyperlocal::{Uri, UnixConnector};
use tokio_core::reactor::Core;

fn run() -> Result<()> {
    let mut core = Core::new()?;
    let handle = core.handle();
    let client = Client::configure()
        .connector(UnixConnector::new(handle))
        .build(&core.handle());
    let work = client
        .get(Uri::new("test.sock", "/").into())
        .and_then(|res| {
            println!("Response: {}", res.status());
            println!("Headers: \n{}", res.headers());

            res.body().for_each(|chunk| {
                io::stdout().write_all(&chunk).map_err(From::from)
            })
        })
        .map(|_| {
            println!("\n\nDone.");
        });

    core.run(work)
}

fn main() {
    run().unwrap()
}
```

Doug Tangren (softprops) 2015-2017

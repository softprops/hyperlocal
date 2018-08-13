# hyperlocal [![Build Status](https://travis-ci.org/softprops/hyperlocal.svg?branch=master)](https://travis-ci.org/softprops/hyperlocal) [![Coverage Status](https://coveralls.io/repos/github/softprops/hyperlocal/badge.svg)](https://coveralls.io/github/softprops/hyperlocal) [![crates.io](https://img.shields.io/crates/v/hyperlocal.svg)](https://crates.io/crates/hyperlocal) [![docs.rs](https://docs.rs/hyperlocal/badge.svg)](https://docs.rs/hyperlocal) [![Master API docs](https://img.shields.io/badge/docs-master-green.svg)](https://softprops.github.io/hyperlocal)

> [hyper](https://github.com/hyperium/hyper) client and server bindings for [unix domain sockets](https://github.com/tokio-rs/tokio-uds)

Hyper is a rock solid [rustlang](https://www.rust-lang.org/) HTTP client and server tool kit. [Unix domain sockets](https://en.wikipedia.org/wiki/Unix_domain_socket) provide
a mechanism for host-local interprocess communication. Hyperlocal builds on and complements hyper's interfaces for building unix domain socket HTTP clients and servers.

This is useful for exposing simple HTTP interfaces for your Unix daemons in cases where you want to limit access to the current host, in which case, opening and exposing tcp ports is not needed. Examples of unix daemons that provide this kind of host local interface include, [docker](https://docs.docker.com/engine/misc/), a process container manager.


## install

Add the following to your `Cargo.toml` file

```toml
[dependencies]
hyperlocal = "0.5"
```

## usage

### servers

A typical server can be built with `hyperlocal::server::Http`

note the example below uses a crate called `service_fn` which exists [here](https://github.com/tokio-rs/service-fn) but is not yet published to crates.io

```rust
extern crate hyper;
extern crate hyperlocal;
extern crate futures;

use hyper::{header, Body, Request, Response};
use hyper::service::service_fn;
use std::io;

const PHRASE: &'static str = "It's a Unix system. I know this.";

fn hello(_: Request<Body>) -> impl futures::Future<Item = Response<Body>, Error = io::Error> + Send {
    futures::future::ok(
        Response::builder()
            .header(header::CONTENT_TYPE, "text/plain")
            .header(header::CONTENT_LENGTH, PHRASE.len() as u64)
            .body(PHRASE.into())
            .expect("failed to create response")
    )
}

fn run() -> io::Result<()> {
    let path = "test.sock";
    let svr = hyperlocal::server::Http::new().bind(path, || service_fn(hello))?;
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
use hyper::{Client, rt};
use hyperlocal::{Uri, UnixConnector};

fn main() {
    let client = Client::builder()
        .keep_alive(false) // without this the connection will remain open
        .build::<_, ::hyper::Body>(UnixConnector::new());
    let url = Uri::new("test.sock", "/").into();

    let work = client
        .get(url)
        .and_then(|res| {
            println!("Response: {}", res.status());
            println!("Headers: {:#?}", res.headers());

            res.into_body().for_each(|chunk| {
                io::stdout().write_all(&chunk)
                    .map_err(|e| panic!("example expects stdout is open, error={}", e))
            })
        })
        .map(|_| {
            println!("\n\nDone.");
        })
        .map_err(|err| {
            eprintln!("Error {}", err);
        });

    rt::run(work);
}
```

Doug Tangren (softprops) 2015-2018

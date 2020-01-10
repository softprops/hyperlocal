# `hyperlocal` [![Build Status](https://travis-ci.org/softprops/hyperlocal.svg?branch=master)](https://travis-ci.org/softprops/hyperlocal) [![Coverage Status](https://coveralls.io/repos/github/softprops/hyperlocal/badge.svg)](https://coveralls.io/github/softprops/hyperlocal) [![crates.io](https://img.shields.io/crates/v/hyperlocal.svg)](https://crates.io/crates/hyperlocal) [![docs.rs](https://docs.rs/hyperlocal/badge.svg)](https://docs.rs/hyperlocal) [![Master API docs](https://img.shields.io/badge/docs-master-green.svg)](https://softprops.github.io/hyperlocal)

> [Hyper](https://github.com/hyperium/hyper) client and server bindings for [Unix domain sockets](https://github.com/tokio-rs/tokio/tree/master/tokio-net/src/uds/)

Hyper is a rock solid [Rust](https://www.rust-lang.org/) HTTP client and server toolkit.
[Unix domain sockets](https://en.wikipedia.org/wiki/Unix_domain_socket) provide a mechanism
for host-local interprocess communication. `hyperlocal` builds on and complements Hyper's
interfaces for building Unix domain socket HTTP clients and servers.

This is useful for exposing simple HTTP interfaces for your Unix daemons in cases where you
want to limit access to the current host, in which case, opening and exposing tcp ports is
not needed. Examples of Unix daemons that provide this kind of host local interface include
[Docker](https://docs.docker.com/engine/misc/), a process container manager.


## Installation

Add the following to your `Cargo.toml` file

```toml
[dependencies]
hyperlocal = "0.7-alpha.1"
```

## Usage

### Servers

A typical server can be built with `hyperlocal::server::UnixServerExt`.

```rust
use std::{error::Error, fs, path::Path};

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Response, Server,
};
use hyperlocal::UnixServerExt;

const PHRASE: &'static str = "It's a Unix system. I know this.";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let path = Path::new("/tmp/hyperlocal.sock");

    if path.exists() {
        fs::remove_file(path)?;
    }

    let make_service = make_service_fn(|_| async {
        Ok::<_, hyper::Error>(service_fn(|_req| async {
            Ok::<_, hyper::Error>(Response::new(Body::from(PHRASE)))
        }))
    });

    Server::bind_unix(path)?.serve(make_service).await?;

    Ok(())
}
```

### Clients

You can communicate over HTTP with Unix domain socket servers using Hyper's `Client` interface.
Configure your Hyper client using `Client::builder()`.

Hyper's client interface makes it easy to issue typical HTTP methods like `GET`, `POST`, `DELETE` with factory
methods, `get`, `post`, `delete`, etc. These require an argument that can be tranformed into a `hyper::Uri`.
Since Unix domain sockets aren't represented with hostnames that resolve to ip addresses coupled with network ports,
your standard URL string won't do. Instead, use a `hyperlocal::Uri`, which represents both file path to the domain
socket and the resource URI path and query string.

```rust
use std::error::Error;
use std::path::Path;

use futures_util::stream::TryStreamExt;
use hyper::{Body, Client};
use hyperlocal::{Uri, UnixClientExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let path = Path::new("/tmp/hyperlocal.sock");
    let url = Uri::new(path, "/").into();

    let client = Client::unix();

    let response_body = client.get(url).await?.into_body;
    
    let bytes = response_body
        .try_fold(Vec::default(), |mut buf, bytes| async {
            buf.extend(bytes);
            Ok(buf)
        })
        .await?;

    println!("{}", String::from_utf8(bytes)?);

    Ok(())
}
```

Doug Tangren (softprops) 2015-2018

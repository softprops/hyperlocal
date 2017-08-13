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

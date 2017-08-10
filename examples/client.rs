extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate hyperlocal;

use hyper::Client;
use hyperlocal::UnixSocketConnector;
use std::io;
use futures::Stream;
use futures::Future;
use std::io::Write;

fn main() {
    let path = "unix://test.sock/".parse::<hyper::Uri>().unwrap();
    println!("{}", path.to_string());
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();

    let client = Client::configure()
        .connector(UnixSocketConnector(handle))
        .build(&core.handle());
    let work = client
        .get(path)
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

    core.run(work).unwrap();
}

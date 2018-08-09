extern crate futures;
extern crate hyper;
extern crate hyperlocal;
extern crate tokio_core;

use std::io::{self, Write};

use futures::Future;
use futures::Stream;
use hyper::{rt, Client};
use hyperlocal::{UnixConnector, Uri};

fn main() {
    let client = Client::builder()
        .keep_alive(false)
        .build::<_, ::hyper::Body>(UnixConnector::new());
    let url = Uri::new("test.sock", "/").into();

    let work = client
        .get(url)
        .and_then(|res| {
            println!("Response: {}", res.status());
            println!("Headers: {:#?}", res.headers());

            res.into_body().for_each(|chunk| {
                io::stdout()
                    .write_all(&chunk)
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

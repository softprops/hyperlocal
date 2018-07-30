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

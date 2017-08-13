extern crate hyper;
extern crate hyperlocal;
extern crate futures;
extern crate tokio_service;

use hyper::{Request, Response};
use hyper::header::ContentLength;
use tokio_service::Service;

struct HelloWorld;

const PHRASE: &'static str = "It's a Unix system. I know this.\n";

impl Service for HelloWorld {
    // boilerplate hooking up hyper's server types
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    // The future representing the eventual Response your call will
    // resolve to. This can change to whatever Future you need.
    type Future = futures::future::FutureResult<Self::Response, Self::Error>;

    fn call(&self, _: Request) -> Self::Future {
        // We're currently ignoring the Request
        // And returning an 'ok' Future, which means it's ready
        // immediately, and build a Response with the 'PHRASE' body.
        futures::future::ok(
            Response::new()
                .with_header(ContentLength(PHRASE.len() as u64))
                .with_body(PHRASE),
        )
    }
}

fn main() {
    let path = "test.sock";
    let svr = hyperlocal::server::Http::new()
        .bind(path, || Ok(HelloWorld))
        .unwrap();
    svr.run().unwrap();

}

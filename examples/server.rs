extern crate hyper;
extern crate hyperlocal;
extern crate futures;
extern crate tokio_service;


use hyper::{Request, Response};
use hyper::header::ContentLength;
use tokio_service::Service;

#[derive(Clone)]
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

    fn call(&self, _req: Request) -> Self::Future {
        // We're currently ignoring the Request
        // And returning an 'ok' Future, which means it's ready
        // immediately, and build a Response with the 'PHRASE' body.
        println!("handling req");
        futures::future::ok(
            Response::new()
                .with_header(ContentLength(PHRASE.len() as u64))
                .with_body(PHRASE),
        )
    }
}

fn main() {
    let path = "test.sock";
    let server = hyperlocal::server::Http::bind(path, HelloWorld).unwrap();
    /*server.handle(|_: Request, res: Response| {
        let _ = res.send(b"It's a Unix system. I know this.\n");
    }).unwrap();
    println!("listening @ {}", path);*/

}

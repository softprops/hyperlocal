extern crate hyper;
extern crate hyperlocal;
extern crate futures;
extern crate tokio_service;

use futures::future::FutureResult;
use hyper::{Result, Request, Response};
use hyper::header::{ContentType, ContentLength};
use tokio_service::Service;

const PHRASE: &'static str = "It's a Unix system. I know this.";

struct Hello;

impl Service for Hello {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = FutureResult<Response, hyper::Error>;
    fn call(&self, _req: Request) -> Self::Future {
        futures::future::ok(
            Response::new()
                .with_header(ContentLength(PHRASE.len() as u64))
                .with_header(ContentType::plaintext())
                .with_body(PHRASE),
        )
    }
}

fn run() -> Result<()> {
    let path = "test.sock";
    let svr = hyperlocal::server::Http::new().bind(path, || Ok(Hello))?;
    println!("Listening on unix://{path} with 1 thread.", path = path);
    svr.run()?;
    Ok(())
}

fn main() {
    run().unwrap()
}

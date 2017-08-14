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

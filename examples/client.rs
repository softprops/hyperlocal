#![feature(async_await)]

use std::error::Error;
use std::path::Path;

use futures::TryStreamExt;
use hyper::{Body, Client};
use hyperlocal::{UnixConnector, Uri};

#[hyper::rt::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let path = Path::new("/tmp/hyperlocal.sock");

    let client = Client::builder().build::<_, Body>(UnixConnector::default());

    let url = Uri::new(path, "/").into();

    let response = client.get(url).await?;
    let bytes = response.into_body().try_concat().await?.to_vec();

    println!("{}", String::from_utf8(bytes)?);

    Ok(())
}

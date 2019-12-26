use std::error::Error;
use std::path::Path;

use futures_util::stream::TryStreamExt;
use hyper::{Body, Client};
use hyperlocal::{UnixConnector, Uri};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let path = Path::new("/tmp/hyperlocal.sock");
    let url = Uri::new(path, "/").into();

    let client = Client::builder().build::<_, Body>(UnixConnector::default());

    let response_body = client.get(url).await?.into_body();

    let bytes = response_body
        .try_fold(Vec::default(), |mut v, bytes| {
            v.extend(bytes);
            futures_util::future::ok(v)
        })
        .await?;

    println!("{}", String::from_utf8(bytes)?);

    Ok(())
}

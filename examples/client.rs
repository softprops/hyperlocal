use std::error::Error;

use hyper::body::HttpBody;
use hyper::Client;
use hyperlocal::{UnixClientExt, Uri};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let url = Uri::new("/tmp/hyperlocal.sock", "/").into();

    let client = Client::unix();

    let mut response = client.get(url).await?;

    let mut bytes = Vec::default();

    while let Some(next) = response.data().await {
        let chunk = next?;
        bytes.extend(chunk);
    }

    println!("{}", String::from_utf8(bytes)?);

    Ok(())
}

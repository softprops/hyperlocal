use std::error::Error;

use futures_util::stream::TryStreamExt;
use hyper::Client;
use hyperlocal::{UnixClientExt, Uri};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let url = Uri::new("/tmp/hyperlocal.sock", "/").into();

    let client = Client::unix();

    let response_body = client.get(url).await?.into_body();

    let bytes = response_body
        .try_fold(Vec::default(), |mut v, bytes| async {
            v.extend(bytes);
            Ok(v)
        })
        .await?;

    println!("{}", String::from_utf8(bytes)?);

    Ok(())
}

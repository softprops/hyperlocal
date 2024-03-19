use hyper::{service::service_fn, Response};
use hyper_util::rt::TokioIo;
use std::{error::Error, fs, io::ErrorKind, path::Path};
use tokio::net::UnixListener;

const PHRASE: &str = "It's a Unix system. I know this.\n";

// Adapted from https://hyper.rs/guides/1/server/hello-world/
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let path = Path::new("/tmp/hyperlocal.sock");

    if path.exists() {
        fs::remove_file(path)?;
    }

    let listener = UnixListener::bind(path)?;

    println!("Listening for connections at {}.", path.display());

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        println!("Accepting connection.");

        tokio::task::spawn(async move {
            let svc_fn = service_fn(|_req| async {
                let body = PHRASE.to_string();
                Ok::<_, hyper::Error>(Response::new(body))
            });

            // On linux, serve_connection will return right away with Result::Ok.
            //
            // On OSX, serve_connection will block until the client disconnects,
            // and return Result::Err(hyper::Error) with a source (inner/cause)
            // socket error indicating the client connection is no longer open.
            match hyper::server::conn::http1::Builder::new()
                .serve_connection(io, svc_fn)
                .await
            {
                Ok(()) => {
                    println!("Accepted connection.");
                }
                Err(err) => {
                    let source: Option<&std::io::Error> =
                        err.source().and_then(|s| s.downcast_ref());

                    match source {
                        Some(io_err) if io_err.kind() == ErrorKind::NotConnected => {
                            println!("Client disconnected.");
                        }
                        _ => {
                            eprintln!("Failed to accept connection: {err:?}");
                        }
                    }
                }
            };
        });
    }
}

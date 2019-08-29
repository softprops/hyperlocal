use std::path::Path;

use hyper::server::{Builder, Server};
use tokio_net::uds::{Incoming, UnixListener};

/// ```rust
/// use hyper::{Server, Body, Response, service::{make_service_fn, service_fn}};
/// use hyperlocal::server::UnixServerExt;
///
/// # async {
/// let make_service = make_service_fn(|_| async {
///     Ok::<_, hyper::Error>(service_fn(|_req| async {
///         Ok::<_, hyper::Error>(Response::new(Body::from("It works!")))
///     }))
/// });
///
/// Server::bind_unix("/tmp/hyperlocal.sock").serve(make_service).await?;
/// # Ok::<_, hyper::Error>(())
/// # };
/// ```
pub trait UnixServerExt {
    /// Convenience method for constructing a Server listening on a Unix socket.
    fn bind_unix(path: impl AsRef<Path>) -> Builder<Incoming>;
}

impl UnixServerExt for Server<Incoming, ()> {
    fn bind_unix(path: impl AsRef<Path>) -> Builder<Incoming> {
        let incoming = UnixListener::bind(path.as_ref())
            .unwrap_or_else(|e| panic!("error binding to {}: {}", path.as_ref().display(), e))
            .incoming();
        Server::builder(incoming)
    }
}

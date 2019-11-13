use std::{io, path::Path};

use hyper::server::{Builder, Server};

use self::conn::SocketIncoming;

pub mod conn {
    use std::{io, path::{Path, PathBuf}, pin::Pin, task::{Context, Poll}};

    use futures_core::stream::Stream;
    use hyper::server::accept::Accept;
    use tokio_net::uds::{Incoming, UnixListener, UnixStream};

    /// A stream of connections from binding to a socket.
    #[derive(Debug)]
    pub struct SocketIncoming {
        path: PathBuf,
        listener: Incoming,
    }

    impl SocketIncoming {
        /// Creates a new `SocketIncoming` binding to provided socket path.
        pub fn bind(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
            let listener = UnixListener::bind(path.as_ref())?.incoming();
            let path = path.as_ref().to_owned();

            Ok(SocketIncoming { path, listener })
        }
    }

    impl Accept for SocketIncoming {
        type Conn = UnixStream;
        type Error = io::Error;

        fn poll_accept(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
            Pin::new(&mut self.listener).poll_next(cx)
        }
    }
}

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
/// Server::bind_unix("/tmp/hyperlocal.sock")?.serve(make_service).await?;
/// # Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
/// # };
/// ```
pub trait UnixServerExt {
    /// Convenience method for constructing a Server listening on a Unix socket.
    fn bind_unix(path: impl AsRef<Path>) -> Result<Builder<SocketIncoming>, io::Error>;
}

impl UnixServerExt for Server<SocketIncoming, ()> {
    fn bind_unix(path: impl AsRef<Path>) -> Result<Builder<SocketIncoming>, io::Error> {
        let incoming = SocketIncoming::bind(path)?;
        Ok(Server::builder(incoming))
    }
}

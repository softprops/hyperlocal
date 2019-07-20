use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::{
    Context,
    Poll::{self, *},
};

use hyper::client::connect::{Connect, Connected, Destination};
use tokio_uds::{ConnectFuture, UnixStream};

use super::Uri;

const UNIX_SCHEME: &str = "unix";

/// A type which implements hyper's client connector interface
/// for unix domain sockets
///
/// `UnixConnector` instances expects uri's
/// to be constructued with `hyperlocal::Uri::new()` which produce uris with a `unix://`
/// scheme
///
/// # Examples
///
/// ```rust
/// use hyper::{Body, Client};
/// use hyperlocal::UnixConnector;
///
/// let client = hyper::Client::builder()
///    .build::<_, hyper::Body>(UnixConnector::default());
/// ```
#[derive(Clone, Debug, Default)]
pub struct UnixConnector;

impl Connect for UnixConnector {
    type Transport = UnixStream;
    type Error = io::Error;
    type Future = UnixConnecting;

    fn connect(&self, destination: Destination) -> Self::Future {
        if destination.scheme() != UNIX_SCHEME {
            return UnixConnecting::Error(Some(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid URL, scheme must be unix",
            )));
        }

        if let Some(path) = Uri::socket_path_dest(&destination) {
            return UnixConnecting::Connecting(UnixStream::connect(&path));
        }

        UnixConnecting::Error(Some(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid URL, host must be a hex-encoded path",
        )))
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub enum UnixConnecting {
    Connecting(ConnectFuture),
    Error(Option<io::Error>),
}

impl Future for UnixConnecting {
    type Output = Result<(UnixStream, Connected), io::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        match this {
            UnixConnecting::Connecting(ref mut f) => match Pin::new(f).poll(cx) {
                Ready(Ok(stream)) => Ready(Ok((stream, Connected::new()))),
                Pending => Pending,
                Ready(Err(err)) => Ready(Err(err)),
            },
            UnixConnecting::Error(ref mut e) => {
                Poll::Ready(Err(e.take().expect("polled more than once")))
            }
        }
    }
}

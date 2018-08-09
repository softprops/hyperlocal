//! Hyper client bindings for unix domain sockets

use std::io;

use futures::{Future, IntoFuture};
use futures::future::{self, FutureResult};
use hyper::client::connect::{Connect, Connected, Destination};
use tokio_uds::UnixStream;

use super::Uri;

const UNIX_SCHEME: &str = "unix";

/// A type which implements hyper's client connector interface
/// for unix domain sockets
///
/// `UnixConnector` instances assume uri's
/// constructued with `hyperlocal::Uri::new()` which produce uris with a `unix://`
/// scheme
///
/// # examples
///
/// ```no_run
/// extern crate hyper;
/// extern crate hyperlocal;
///
/// let client = hyper::Client::builder()
///    .build::<_, hyper::Body>(hyperlocal::UnixConnector::new());
/// ```
#[derive(Clone)]
pub struct UnixConnector;

impl UnixConnector {
    pub fn new() -> Self {
        UnixConnector
    }
}

impl Connect for UnixConnector {
    type Transport = UnixStream;
    type Error = io::Error;
    type Future = FutureResult<(UnixStream, Connected), io::Error>;

    fn connect(&self, destination: Destination) -> Self::Future {
        if destination.scheme() != UNIX_SCHEME {
            return future::err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid uri {:?}", destination),
            ));
        }
        match Uri::socket_path_dest(&destination) {
            Some(ref path) => UnixStream::connect(path)
                                .wait() // We have to block because we
                                .map(|s| (s, Connected::new()))
                                .into_future(),
            _ => future::err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid uri {:?}", destination),
            )),
        }
    }
}


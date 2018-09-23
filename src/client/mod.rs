//! Hyper client bindings for unix domain sockets

// Std lib
use std::io;

// Third party
use futures::{Async, Future, Poll};
use hyper::client::connect::{Connect, Connected, Destination};
use tokio_uds::{ConnectFuture as StreamConnectFuture, UnixStream};

use super::Uri;

const UNIX_SCHEME: &str = "unix";

/// A type which implements hyper's client connector interface
/// for unix domain sockets
///
/// `UnixConnector` instances expects uri's
/// to be constructued with `hyperlocal::Uri::new()` which produce uris with a `unix://`
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
    type Future = ConnectFuture;

    fn connect(&self, destination: Destination) -> Self::Future {
        ConnectFuture::Start(destination)
    }
}

pub enum ConnectFuture {
    Start(Destination),
    Connect(StreamConnectFuture),
}

impl Future for ConnectFuture {
    type Item = (UnixStream, Connected);
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            let next_state = match self {
                ConnectFuture::Start(destination) => {
                    if destination.scheme() != UNIX_SCHEME {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!("Invalid uri {:?}", destination),
                        ));
                    }

                    let path = match Uri::socket_path_dest(&destination) {
                        Some(path) => path,

                        None => {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidInput,
                                format!("Invalid uri {:?}", destination),
                            ))
                        }
                    };

                    ConnectFuture::Connect(UnixStream::connect(&path))
                }

                ConnectFuture::Connect(f) => match f.poll() {
                    Ok(Async::Ready(stream)) => return Ok(Async::Ready((stream, Connected::new()))),
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Err(err) => return Err(err),
                },
            };

            *self = next_state;
        }
    }
}

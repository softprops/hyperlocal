//! hyperlocal provides [hyper](http://github.com/hyperium/hyper) client and server bindings
//! for [unix domain sockets](http://rust-lang-nursery.github.io/unix-socket/doc/v0.5.0/unix_socket/)
//!
//! See the `UnixSocketConnector` docs for how to configure hyper clients and the `UnixSocketServer` docs
//! for how to configure hyper servers
extern crate futures;
extern crate hyper;
extern crate tokio_uds;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_service;

use std::io;
use std::path::Path;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use futures::future::{Future, FutureResult};
use futures::stream::Stream;
use hyper::{Request, Response, Uri};
use hyper::server::Http as HyperHttp;
use tokio_uds::{UnixListener, UnixStream};
use tokio_core::reactor::{Core, Handle};
use tokio_service::Service;

const UNIX_SCHEME: &str = "unix";

pub struct UnixSocketConnector(pub Handle);

impl Service for UnixSocketConnector {
    type Request = Uri;
    type Response = UnixStream;
    type Error = io::Error;
    type Future = FutureResult<UnixStream, io::Error>;

    fn call(&self, uri: Uri) -> Self::Future {
        if uri.scheme() != Some(UNIX_SCHEME) {
            return futures::future::err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid uri {}", uri),
            ));
        }
        match UnixStream::connect(uri.to_string(), &self.0) {
            Ok(stream) => futures::future::ok(stream),
            Err(err) => futures::future::err(err),
        }
    }
}

pub struct Http;

impl Http {
    pub fn bind<P: AsRef<Path>, S, Bd, B>(path: P, service: S) -> io::Result<()>
    where
        B: AsRef<[u8]> + 'static,
        S: Service<Request = Request, Response = Response<Bd>, Error = ::hyper::Error> + 'static + Clone,
        Bd: Stream<Item = B, Error = ::hyper::Error> + 'static,
    {
        let mut core = Core::new()?;
        let handle = core.handle();
        let listener = UnixListener::bind(path, &handle)?;
        let server = listener
            .incoming()
            .for_each(move |(sock, _)| {
                HyperHttp::new().bind_connection(
                    &handle,
                    sock,
                    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0)),
                    service.clone(),
                );
                Ok(())
            })
            .map_err(|_| ());
        core.run(server).map_err(|_| ());
        Ok(())
    }
}

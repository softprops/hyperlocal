//! Hyper server bindings for unix domain sockets

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tokio_uds::{UnixListener, UnixStream};
use tokio_core::reactor::{Core, Handle};
use std::io;
use futures::stream::Stream;
use futures::future::Future;
use hyper::{Request, Response, Uri};
use hyper::server::Http as HyperHttp;
use tokio_service::Service;
use std::path::Path;

/// A type that provides a factory interface for creating
/// unix socket based hyper Servers
///
/// # examples
///
/// ```no_run
/// extern crate hyper;
/// extern crate hyperlocal;
///
/// //let server = hyperlocal::Http::bind(
///  // "path/to/socket",
///  //  || HelloWorld
/// //)
///
/// ```
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

//! Hyper server bindings for unix domain sockets

// Std lib
use std::error;
use std::fmt;
use std::io;
use std::os::unix::net::SocketAddr;
use std::path::Path;

// Third party
use futures::future::Future;
use futures::stream::Stream;
use hyper::body::Payload;
use hyper::server::conn::Http as HyperHttp;
use hyper::service::{NewService, Service};
use hyper::Body;
use tokio_core::reactor::Core;
use tokio_uds::UnixListener;

/// An instance of a server created through `Http::bind`.
//
/// This structure is used to create instances of Servers to spawn off tasks
/// which handle a connection to an HTTP server.
pub struct Server<S>
where
    S: NewService<ReqBody = Body> + Send + 'static,
{
    new_service: S,
    core: Core,
    listener: UnixListener,
}

impl<S> Server<S>
where
    S: NewService<ReqBody = Body, ResBody = Body, Error = io::Error> + Send + Sync + 'static,
    S::InitError: fmt::Display,
    <S::Service as Service>::Future: Send,
{
    /// Return the of the underlying socket address this server is listening on
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    /// Start listening for incomming connections
    pub fn run(self) -> io::Result<()> {
        let Server {
            new_service,
            mut core,
            listener,
        } = self;

        let server = listener.incoming().for_each(move |sock| {
            new_service
                .new_service()
                .map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("failed to create service: {}", e),
                    )
                })
                .and_then(|service| {
                    HyperHttp::new()
                        .serve_connection(sock, service)
                        .map_err(|e| {
                            io::Error::new(
                                io::ErrorKind::Other,
                                format!("failed to serve connection: {}", e),
                            )
                        })
                })
        });

        core.run(server)
    }
}

/// A type that provides a factory interface for creating
/// unix socket based Servers
///
/// # examples
///
/// ```no_run
/// extern crate hyper;
/// extern crate hyperlocal;
///
/// //let server = hyperlocal::Http::new().bind(
///  // "path/to/socket",
///  //  || Ok(HelloWorld)
/// //)
///
/// ```
#[derive(Clone)]
pub struct Http;

impl Http {
    /// Creates a new instance of the HTTP protocol, ready to spawn a server or
    /// start accepting connections.
    pub fn new() -> Self {
        Http
    }

    /// Binds a new server instance to a unix domain socket path
    /// If the provided path exists this method will yield an error
    pub fn bind<P, S, B>(&self, path: P, new_service: S) -> io::Result<Server<S>>
    where
        P: AsRef<Path>,
        S: NewService<ReqBody = Body, ResBody = B> + Send + 'static,
        S::Error: Into<Box<error::Error + Send + Sync>>,
        S::Service: Send,
        <S::Service as Service>::Future: Send + 'static,
        B: Payload,
    {
        let core = Core::new()?;
        let listener = UnixListener::bind(path.as_ref())?;

        Ok(Server {
            core,
            listener,
            new_service,
        })
    }
}

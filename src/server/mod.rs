//! Hyper server bindings for unix domain sockets

// Std lib
use std::error;
use std::fmt;
use std::io;
use std::os::unix::net::{SocketAddr, UnixListener as StdUnixListener};
use std::path::Path;

// Third party
use futures01::{Async, Future, Poll, Stream};
use hyper::body::Payload;
use hyper::server::conn::{Connection as HyperConnection, Http as HyperHttp};
use hyper::service::{NewService, Service};
use hyper::Body;
use tokio::{reactor::Handle, runtime::Runtime};
use tokio_uds::{Incoming as UnixIncoming, UnixListener, UnixStream};

/// An instance of a unix domain socket server created through `Server::bind`.
///
/// # Examples
///
/// ```rust
/// extern crate hyper;
/// extern crate hyperlocal;
///
/// use hyper::service::service_fn;
/// use hyperlocal::server::Server;
///
/// # if let Err(err) = std::fs::remove_file("hyperlocal_test_echo_server_1.sock") {
/// #   if err.kind() != std::io::ErrorKind::NotFound {
/// #     panic!("{}", err)
/// #   }
/// # }
/// #
/// let echo_server = Server::bind(
///    "hyperlocal_test_echo_server_1.sock",
///    || service_fn(|req| Ok::<_, hyper::Error>(hyper::Response::new(req.into_body())))
/// ).unwrap();
/// ```
pub struct Server<S> {
    serve: Serve<S>,
}

impl<S> Server<S> {
    /// Binds a new server instance to a unix domain socket path.
    ///
    /// If the provided path exists, this method will return an error.
    pub fn bind<P>(path: P, new_service: S) -> io::Result<Server<S>>
    where
        P: AsRef<Path>,
        S: NewService<ReqBody = Body>,
        S::ResBody: Payload,
        S::Service: Send,
        S::Error: Into<Box<error::Error + Send + Sync>>,
        <S::Service as Service>::Future: Send + 'static,
    {
        let protocol = Http::new();
        let serve = protocol.serve_path(path, new_service)?;
        Ok(Server { serve })
    }

    /// Return the local address of the underlying socket that this server is listening on.
    pub fn local_addr(&self) -> &SocketAddr {
        self.serve.incoming.local_addr()
    }

    /// Start a new tokio runtime, and drive this server on it.
    pub fn run(self) -> io::Result<()>
    where
        S: NewService<ReqBody = Body> + Send + 'static,
        S::Future: Send + 'static,
        S::Service: Send,
        S::InitError: fmt::Display,
        <S::Service as Service>::ResBody: Payload,
        <S::Service as Service>::Future: Send + 'static,
    {
        let runtime = Runtime::new()?;

        runtime.block_on_all(self.serve.for_each(|connecting| {
            connecting
                .map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("failed to serve connection: {}", e),
                    )
                })
                .and_then(|connection| {
                    connection.map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            format!("failed to serve connection: {}", e),
                        )
                    })
                })
        }))
    }
}

/// A stream mapping incoming connections to new services.
///
/// Yields `Connecting`s that are futures that should be put on a reactor.
pub struct Serve<S> {
    incoming: Incoming,
    new_service: S,
    protocol: HyperHttp,
}

impl<S> Stream for Serve<S>
where
    S: NewService<ReqBody = Body>,
{
    type Item = Connecting<S::Future>;
    type Error = <UnixIncoming as Stream>::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.incoming.poll()? {
            Async::Ready(Some(stream)) => {
                let service_future = self.new_service.new_service();
                Ok(Async::Ready(Some(Connecting {
                    service_future,
                    stream: Some(stream),
                    protocol: self.protocol.clone(),
                })))
            }
            Async::Ready(None) => Ok(Async::Ready(None)),
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}

/// A future building a new `Service` to a `Connection`.
///
/// Wraps the future returned from `NewService` into one that returns a `Connection`.
pub struct Connecting<F> {
    service_future: F,
    stream: Option<UnixStream>,
    protocol: HyperHttp,
}

impl<F> Future for Connecting<F>
where
    F: Future,
    F::Item: Service<ReqBody = Body>,
    <F::Item as Service>::ResBody: Payload,
    <F::Item as Service>::Future: Send + 'static,
{
    type Item = HyperConnection<UnixStream, F::Item>;
    type Error = F::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let service = match self.service_future.poll()? {
            Async::Ready(service) => service,
            Async::NotReady => return Ok(Async::NotReady),
        };
        let stream = self.stream.take().expect("polled after complete");
        Ok(Async::Ready(
            self.protocol.serve_connection(stream, service),
        ))
    }
}

/// A lower-level method of creating a unix domain socket server.
///
/// This structure is used to manage connections yourself. If you don't need to do this,
/// consider using the higher-level `Server::bind` API.
///
/// # Examples
///
/// ```rust
/// extern crate hyper;
/// extern crate hyperlocal;
///
/// use std::os::unix::net::UnixListener;
/// use hyper::{Response, rt::{Future, Stream}, service::service_fn};
/// use hyperlocal::server::{Http, Incoming};
///
/// # if let Err(err) =  std::fs::remove_file("hyperlocal_test_echo_server_2.sock") {
/// #   if err.kind() != std::io::ErrorKind::NotFound {
/// #     panic!("{}", err);
/// #   }
/// # }
/// #
/// let listener = UnixListener::bind("hyperlocal_test_echo_server_2.sock").unwrap();
/// let incoming = Incoming::from_std(listener, &Default::default()).unwrap();
/// let serve = Http::new().serve_incoming(
///   incoming,
///   move || service_fn(
///     |req| Ok::<_, hyper::Error>(Response::new(req.into_body()))
///   )
///  );
///
/// let server = serve.for_each(|connecting| {
///     connecting
///     .then(|connection| {
///         let connection = connection.unwrap();
///         Ok::<_, hyper::Error>(connection)
///     })
///     .flatten()
///     .map_err(|err| {
///         std::io::Error::new(
///             std::io::ErrorKind::Other,
///             format!("failed to serve connection: {}", err),
///         )
///     })
/// });
/// ```
#[derive(Clone)]
pub struct Http {
    inner: HyperHttp,
}

impl Http {
    /// Creates a new instance of the HTTP protocol, ready to spawn a server or
    /// start accepting connections.
    pub fn new() -> Self {
        Http::from_hyper(HyperHttp::new())
    }

    /// Creates a new instance of the HTTP protocol using the given hyper `Http`,
    /// ready to spawn a server or start accepting connections.
    pub fn from_hyper(hyper_http: HyperHttp) -> Self {
        Http { inner: hyper_http }
    }

    /// Bind the provided `path` with the default `Handle` and return `Serve`.
    ///
    /// This method will bind the unix domain socket path provided with
    /// a new UDS listener ready to accept connections. Each connection will be
    /// processed with the `new_service` object provided, creating a new service per
    /// connection.
    ///
    /// If the provided path already exists, this method will return an error.
    pub fn serve_path<P, S>(&self, path: P, new_service: S) -> io::Result<Serve<S>>
    where
        P: AsRef<Path>,
        S: NewService<ReqBody = Body>,
        S::ResBody: Payload,
        S::Service: Send,
        S::Error: Into<Box<error::Error + Send + Sync>>,
        <S::Service as Service>::Future: Send + 'static,
    {
        let incoming = Incoming::new(path, None)?;
        Ok(self.serve_incoming(incoming, new_service))
    }

    /// Bind the provided `path` with the `Handle` and return `Serve`.
    ///
    /// This method will bind the unix domain socket path provided with
    /// a new UDS listener ready to accept connections. Each connection will be
    /// processed with the `new_service` object provided, creating a new service per
    /// connection.
    ///
    /// If the provided path already exists, this method will return an error.
    pub fn serve_path_handle<P, S>(
        &self,
        path: P,
        handle: &Handle,
        new_service: S,
    ) -> io::Result<Serve<S>>
    where
        P: AsRef<Path>,
        S: NewService<ReqBody = Body> + Send + 'static,
        S::ResBody: Payload,
        S::Service: Send,
        S::Error: Into<Box<error::Error + Send + Sync>>,
        <S::Service as Service>::Future: Send + 'static,
    {
        let incoming = Incoming::new(path, Some(handle))?;
        Ok(self.serve_incoming(incoming, new_service))
    }

    /// Bind the provided stream of incoming `UnixStream` objects with a `NewService`.
    pub fn serve_incoming<S>(&self, incoming: Incoming, new_service: S) -> Serve<S>
    where
        S: NewService<ReqBody = Body>,
        S::ResBody: Payload,
        S::Error: Into<Box<::std::error::Error + Send + Sync>>,
    {
        Serve {
            incoming,
            new_service,
            protocol: self.inner.clone(),
        }
    }
}

impl From<HyperHttp> for Http {
    fn from(hyper_http: HyperHttp) -> Self {
        Http::from_hyper(hyper_http)
    }
}

/// A stream of unix domain socket connections.
pub struct Incoming {
    inner: UnixIncoming,
    local_addr: SocketAddr,
}

impl Incoming {
    /// Bind a listener to the provided `path` with the provided `Handle`.
    ///
    /// If the `Handle` is `None`, the current runtime handle is used.
    pub fn new<P>(path: P, handle: Option<&Handle>) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let listener = StdUnixListener::bind(path)?;
        match handle {
            Some(handle) => Incoming::from_std(listener, handle),
            None => {
                let handle = Handle::default();
                Incoming::from_std(listener, &handle)
            }
        }
    }

    /// Wrap the provided already-bound listener in a `tokio_uds` listener using the provided `Handle`.
    pub fn from_std(listener: StdUnixListener, handle: &Handle) -> io::Result<Self> {
        let listener = UnixListener::from_std(listener, handle)?;
        let local_addr = listener.local_addr()?;
        let inner = listener.incoming();
        Ok(Incoming { inner, local_addr })
    }

    /// Get the local address bound to this listener.
    pub fn local_addr(&self) -> &SocketAddr {
        &self.local_addr
    }
}

impl Stream for Incoming {
    type Item = <UnixIncoming as Stream>::Item;
    type Error = <UnixIncoming as Stream>::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.inner.poll()
    }
}

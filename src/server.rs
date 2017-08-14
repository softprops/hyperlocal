//! Hyper server bindings for unix domain sockets

use std::marker::PhantomData;
use std::path::Path;

use futures::future::Future;
use futures::stream::Stream;
use hyper::{Request, Response};
use hyper::server::Http as HyperHttp;
use tokio_core::reactor::Core;
use tokio_service::NewService;
use tokio_uds::UnixListener;

/// An instance of a server created through `Http::bind`.
//
/// This structure is used to create instances of Servers to spawn off tasks
/// which handle a connection to an HTTP server.
pub struct Server<S, B>
where
    B: Stream<Error = ::hyper::Error>,
    B::Item: AsRef<[u8]>,
{
    protocol: HyperHttp<B::Item>,
    new_service: S,
    core: Core,
    listener: UnixListener,
}

impl<S, B> Server<S, B>
where
    S: NewService<Request = Request, Response = Response<B>, Error = ::hyper::Error>
        + Send
        + Sync
        + 'static,
    B: Stream<Error = ::hyper::Error> + 'static,
    B::Item: AsRef<[u8]>,
{
    pub fn run(self) -> ::hyper::Result<()> {
        let Server {
            protocol,
            new_service,
            mut core,
            listener,
            ..
        } = self;
        let handle = core.handle();
        let server = listener
            .incoming()
            .for_each(move |(sock, _)| {
                protocol.bind_connection(
                    &handle,
                    sock,
                    ([127, 0, 0, 1], 0).into(),
                    new_service.new_service()?,
                );
                Ok(())
            })
            .map_err(|_| ());
        core.run(server);
        Ok(())
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
///  //  || HelloWorld
/// //)
///
/// ```
pub struct Http<B = ::hyper::Chunk> {
    _marker: PhantomData<B>,
}

impl<B> Clone for Http<B> {
    fn clone(&self) -> Http<B> {
        Http { ..*self }
    }
}

impl<B: AsRef<[u8]> + 'static> Http<B> {
    /// Creates a new instance of the HTTP protocol, ready to spawn a server or
    /// start accepting connections.
    pub fn new() -> Http<B> {
        Http { _marker: PhantomData }
    }

    /// binds a new server instance to a unix domain socket path
    pub fn bind<P, S, Bd>(&self, path: P, new_service: S) -> ::hyper::Result<Server<S, Bd>>
    where
        P: AsRef<Path>,
        S: NewService<Request = Request, Response = Response<Bd>, Error = ::hyper::Error>
            + Send
            + Sync
            + 'static,
        Bd: Stream<Item = B, Error = ::hyper::Error> + 'static,
    {
        let core = Core::new()?;
        let handle = core.handle();
        let listener = UnixListener::bind(path.as_ref(), &handle)?;

        Ok(Server {
            protocol: HyperHttp::new(),
            new_service: new_service,
            core: core,
            listener: listener,
        })
    }
}

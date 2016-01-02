extern crate hyper;
extern crate unix_socket;
extern crate url;

use hyper::client::IntoUrl;
use hyper::net::{NetworkConnector, NetworkStream, NetworkListener};
use hyper::Server;
use std::io::{self, Read, Write};
use std::path::Path;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;
use unix_socket::{UnixListener, UnixStream};
use url::{parse_path, Host, Url, SchemeData, RelativeSchemeData};
use url::ParseError as UrlError;

const UNIX_SCHEME: &'static str = "unix";

/// A type which implements NetworkConnector and NetworkStream interfaces
/// for unix domain sockets
pub struct UnixConnector;

// we wrap because we can't impl traits not defined in this crate
pub struct UnixSocketStream(pub UnixStream);

impl Clone for UnixSocketStream {
    #[inline]
    fn clone(&self) -> UnixSocketStream {
        UnixSocketStream(self.0.try_clone().unwrap())
    }
}

impl NetworkConnector for UnixConnector {
    type Stream = UnixSocketStream;

    fn connect(&self, host: &str, _: u16, scheme: &str) -> hyper::Result<UnixSocketStream> {
        Ok(try!(match scheme {
            unix if unix == UNIX_SCHEME => {
                Ok(UnixSocketStream(try!(UnixStream::connect(host))))
            },
            _ => {
                Err(io::Error::new(io::ErrorKind::InvalidInput,
                                   "Invalid scheme for unix"))
            }
        }))
    }
}

impl NetworkStream for UnixSocketStream {
    #[inline]
    fn peer_addr(&mut self) -> io::Result<SocketAddr> {
        self.0.peer_addr().map(|_|{
            SocketAddr::V4(
                SocketAddrV4::new(
                    Ipv4Addr::new(0, 0, 0, 0),
                    0
                )
            )
        })
    }

    #[inline]
    fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.0.set_read_timeout(dur)
    }

    #[inline]
    fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.0.set_write_timeout(dur)
    }
}


impl Read for UnixSocketStream {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}

impl Write for UnixSocketStream {
    #[inline]
    fn write(&mut self, msg: &[u8]) -> std::io::Result<usize> {
        self.0.write(msg)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

/// A type which implmements hyper's IntoUrl interface
/// for unix domain sockets
pub struct DomainUrl<'a> {
    /// path to domain socket
    socket: &'a str,
    /// url path including leading slash, path, and query string
    path: &'a str
}

impl<'a> DomainUrl<'a> {
    pub fn new(socket: &'a str, path: &'a str) -> DomainUrl<'a> {
        DomainUrl {
            socket: socket, path: path
        }
    }
}

impl<'a> IntoUrl for DomainUrl<'a> {
    fn into_url(self) -> Result<Url, UrlError> {
        let (path, query, fragment) = try!(parse_path(self.path));
        Ok(Url {
            scheme: UNIX_SCHEME.to_owned(),
            scheme_data: SchemeData::Relative(
                RelativeSchemeData {
                    username: "".to_owned(),
                    password: None,
                    host: Host::Domain(self.socket.to_owned()),
                    port: Some(0),
                    default_port: None,
                    path: path
                }),
            query: query,
            fragment: fragment
        })
    }
}

#[derive(Debug)]
pub struct UnixSocketListener(pub UnixListener);

impl Drop for UnixSocketListener {
    fn drop(&mut self) {
        println!("dropping socker listener. todo delete socket path");
    }
}

impl Clone for UnixSocketListener {
    #[inline]
    fn clone(&self) -> UnixSocketListener {
        UnixSocketListener(self.0.try_clone().unwrap())
    }
}

impl UnixSocketListener {
    /// Start listening to an address over HTTP.
    pub fn new<P: AsRef<Path>>(addr: P) -> hyper::Result<UnixSocketListener> {
        Ok(UnixSocketListener(try!(UnixListener::bind(addr))))
    }
}

impl NetworkListener for UnixSocketListener {
    type Stream = UnixSocketStream;

    #[inline]
    fn accept(&mut self) -> hyper::Result<UnixSocketStream> {
        Ok(UnixSocketStream(try!(self.0.accept()).0))
    }

    #[inline]
    fn local_addr(&mut self) -> io::Result<SocketAddr> {
        self.0.local_addr().map(|_| {
            SocketAddr::V4(
                SocketAddrV4::new(
                    Ipv4Addr::new(0, 0, 0, 0), 0
                )
            )
        })
    }
}

/// A type that provides a factory interface for creating
/// unix socket based hyper Servers
pub struct UnixSocketServer;

impl UnixSocketServer {
    /// creates a new hyper Server from a unix socket path
    pub fn new<P: AsRef<Path>>(p: P) -> hyper::Result<Server<UnixSocketListener>> {
        UnixSocketListener::new(p).map(Server::new)
    }
}

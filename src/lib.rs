extern crate hyper;
extern crate unix_socket;
extern crate url;

use hyper::client::IntoUrl;
use hyper::net::{NetworkConnector};
use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::time::Duration;
use unix_socket::UnixStream;
use url::{parse_path, Host, Url, SchemeData, RelativeSchemeData};
use url::ParseError as UrlError;

pub struct SocketConnector;

pub struct SocketStream(pub UnixStream);

impl NetworkConnector for SocketConnector {
    type Stream = SocketStream;

    fn connect(&self, host: &str, _: u16, scheme: &str) -> hyper::Result<SocketStream> {
        Ok(try!(match scheme {
            "unix" => {
                Ok(SocketStream(try!(UnixStream::connect(host))))
            },
            _ => {
                Err(io::Error::new(io::ErrorKind::InvalidInput,
                                   "Invalid scheme for Http"))
            }
        }))
    }
}

impl hyper::net::NetworkStream for SocketStream {
    #[inline]
    fn peer_addr(&mut self) -> io::Result<SocketAddr> {
        //                    self.0.peer_addr()
        Err(io::Error::new(io::ErrorKind::InvalidInput, "unix domain sockets do not apply here"))
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


impl Read for SocketStream {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}

impl Write for SocketStream {
    #[inline]
    fn write(&mut self, msg: &[u8]) -> std::io::Result<usize> {
        self.0.write(msg)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

pub struct DomainUrl<'a> {
    socket: &'a str,
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
            scheme: "unix".to_owned(),
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

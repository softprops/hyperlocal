#![deny(missing_debug_implementations, unreachable_pub, rust_2018_idioms)]

//! `hyperlocal` provides [Hyper](http://github.com/hyperium/hyper) bindings
//! for [Unix domain sockets](https://github.com/tokio-rs/tokio/tree/master/tokio-net/src/uds/).
//!
//! See the [`hyperlocal::UnixConnector`](crate::client::UnixConnector) docs for how to
//! configure clients and the [`hyperlocal::UnixServerExt`](crate::server::UnixServerExt)
//! docs for how to configure servers.

use std::borrow::Cow;
use std::io;
use std::path::{Path, PathBuf};

use hex::FromHex;
use hyper::Uri as HyperUri;

pub mod client;
pub use client::UnixConnector;

//pub mod server;
//pub use server::UnixServerExt;

/// A type which implements `Into` for Hyper's  `hyper::Uri`
/// type targeting Unix domain sockets.
///
/// You can use this with any of the HTTP factory
/// methods on hyper's Client interface and for
/// creating requests.
///
/// ```
/// let url: hyper::Uri = hyperlocal::Uri::new(
///     "/path/to/socket", "/urlpath?key=value"
/// ).into();
///
/// let req = hyper::Request::get(url).body(()).unwrap();
/// ```
#[derive(Debug)]
pub struct Uri<'a> {
    /// url path including leading slash, path, and query string
    encoded: Cow<'a, str>,
}

impl<'a> Into<HyperUri> for Uri<'a> {
    fn into(self) -> HyperUri {
        self.encoded.as_ref().parse().unwrap()
    }
}

impl<'a> Uri<'a> {
    /// Productes a new `Uri` from path to domain socket and request path.
    /// request path should include a leading slash
    pub fn new(socket: impl AsRef<Path>, path: &'a str) -> Self {
        let host = hex::encode(socket.as_ref().to_string_lossy().as_bytes());
        let host_str = format!("unix://{}:0{}", host, path);
        Uri {
            encoded: Cow::Owned(host_str),
        }
    }

    fn parse_socket_path(scheme: &str, host: &str) -> Result<PathBuf, io::Error> {
        if scheme != "unix" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid URL, scheme must be unix",
            ));
        }

        let bytes = Vec::from_hex(host).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid URL, host must be a hex-encoded path",
            )
        })?;

        Ok(PathBuf::from(String::from_utf8_lossy(&bytes).into_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::Uri as HyperUri;

    #[test]
    fn test_unix_uri_into_hyper_uri() {
        let unix: HyperUri = Uri::new("foo.sock", "/").into();
        let expected: HyperUri = "unix://666f6f2e736f636b:0/".parse().unwrap();
        assert_eq!(unix, expected);
    }

    #[test]
    fn test_hex_encoded_unix_uri() {
        let uri: HyperUri = "unix://666f6f2e736f636b:0/".parse().unwrap();

        let path = Uri::parse_socket_path(uri.scheme_str().unwrap(), uri.host().unwrap()).unwrap();
        assert_eq!(path, PathBuf::from("foo.sock"));
    }

    #[test]
    fn test_hex_encoded_non_unix_uri() {
        let uri: HyperUri = "http://666f6f2e736f636b:0/".parse().unwrap();

        let err =
            Uri::parse_socket_path(uri.scheme_str().unwrap(), uri.host().unwrap()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn test_non_hex_encoded_non_unix_uri() {
        let uri: HyperUri = "http://example.org".parse().unwrap();

        let err =
            Uri::parse_socket_path(uri.scheme_str().unwrap(), uri.host().unwrap()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn test_non_hex_encoded_unix_uri() {
        let uri: HyperUri = "unix://example.org".parse().unwrap();

        let err =
            Uri::parse_socket_path(uri.scheme_str().unwrap(), uri.host().unwrap()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }
}

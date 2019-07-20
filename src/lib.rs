#![deny(missing_debug_implementations, unreachable_pub, rust_2018_idioms)]

//! `hyperlocal` provides [Hyper](http://github.com/hyperium/hyper) bindings
//! for [Unix domain sockets](http://github.com/tokio-rs/tokio/tree/master/tokio-uds/).
//!
//! See the [`hyperlocal::UnixConnector`](crate::client::UnixConnector) docs for how to
//! configure clients and the [`hyperlocal::UnixServerExt`](crate::server::UnixServerExt)
//! docs for how to configure servers.

use std::borrow::Cow;
use std::path::Path;

use hex::FromHex;
use hyper::{client::connect::Destination, Uri as HyperUri};

pub mod client;
pub use client::UnixConnector;

pub mod server;
pub use server::UnixServerExt;

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
    pub fn new<P>(socket: P, path: &'a str) -> Self
    where
        P: AsRef<Path>,
    {
        let host = hex::encode(socket.as_ref().to_string_lossy().as_bytes());
        let host_str = format!("unix://{}:0{}", host, path);
        Uri {
            encoded: Cow::Owned(host_str),
        }
    }

    // fixme: would like to just use hyper::Result and hyper::error::UriError here
    // but UriError its not exposed for external use
    fn socket_path(uri: &HyperUri) -> Option<String> {
        uri.host()
            .iter()
            .filter_map(|host| {
                Vec::from_hex(host)
                    .ok()
                    .map(|raw| String::from_utf8_lossy(&raw).into_owned())
            })
            .next()
    }

    fn socket_path_dest(dest: &Destination) -> Option<String> {
        format!("unix://{}", dest.host())
            .parse()
            .ok()
            .and_then(|uri| Self::socket_path(&uri))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::Uri as HyperUri;

    #[test]
    fn unix_uris_into_hyper_uris() {
        let unix: HyperUri = Uri::new("foo.sock", "/").into();
        let expected: HyperUri = "unix://666f6f2e736f636b:0/".parse().unwrap();
        assert_eq!(unix, expected);
    }

    #[test]
    fn unix_uris_resolve_socket_path() {
        let path = Uri::socket_path(&"unix://666f6f2e736f636b:0/".parse().unwrap()).unwrap();
        let expected = "foo.sock";
        assert_eq!(path, expected);
    }

    #[test]
    fn connector_rejects_non_unix_uris() {
        assert_eq!(
            None,
            Uri::socket_path(&"http://google.com".parse().unwrap())
        );
    }

    #[test]
    fn connector_rejects_hand_crafted_unix_uris() {
        assert_eq!(
            None,
            Uri::socket_path(&"unix://google.com".parse().unwrap())
        );
    }
}

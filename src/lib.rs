#![deny(missing_debug_implementations, unreachable_pub, rust_2018_idioms)]

//! `hyperlocal` provides [Hyper](http://github.com/hyperium/hyper) bindings
//! for [Unix domain sockets](https://github.com/tokio-rs/tokio/tree/master/tokio-net/src/uds/).
//!
//! See the [`hyperlocal::UnixConnector`](crate::client::UnixConnector) docs for
//! how to configure clients and the
//! [`hyperlocal::UnixServerExt`](crate::server::UnixServerExt) docs for how to
//! configure servers.

mod client;
pub use client::UnixConnector;

mod server;
pub use server::UnixServerExt;

mod uri;
pub use uri::Uri;

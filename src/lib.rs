#![deny(
    missing_debug_implementations,
    unreachable_pub,
    rust_2018_idioms,
    missing_docs
)]
#![warn(clippy::all, clippy::pedantic)]

//! `hyperlocal` provides [Hyper](http://github.com/hyperium/hyper) bindings
//! for [Unix domain sockets](https://github.com/tokio-rs/tokio/tree/master/tokio-net/src/uds/).
//!
//! See the [`UnixClientExt`] docs for
//! how to configure clients.
//!
//! See the
//! [`UnixServerExt`] docs for how to
//! configure servers.
//!
//! The [`UnixConnector`] can be used in the [`hyper::Client`] builder
//! interface, if required.
//!
//! # Features
//!
//! - Client- enables the client extension trait and connector.
//!
//! - Server- enables the server extension trait.

mod uri;
pub use uri::Uri;

macro_rules! attr_each {
    (#[$meta:meta] $($item:item)*) => {
        $(
            #[$meta]
            $item
        )*
    }
}

attr_each! {
    #[cfg(feature = "client")]

    mod client;
    pub use client::{UnixClientExt, UnixConnector};
}

attr_each! {
    #[cfg(feature = "server")]

    mod server;
    pub use server::UnixServerExt;
    pub use server::conn::SocketIncoming;
}

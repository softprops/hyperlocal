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
//! See the [`UnixClientExt`] docs for how to configure clients.
//!
//! See the [`UnixServerExt`] docs for how to configure servers.
//!
//! The [`UnixConnector`] can be used in the [`hyper::Client`] builder
//! interface, if required.
//!
//! # Features
//!
//! By default `hyperlocal` does not enable any [feature flags](https://doc.rust-lang.org/cargo/reference/features.html).
//!
//! The following features are available:
//!
//! - **`client`** — Enables the client extension trait and connector.
//! - **`server`** — Enables the server extension trait.

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::{UnixClientExt, UnixConnector};

#[cfg(feature = "server")]
mod server;
#[cfg(feature = "server")]
pub use server::UnixServerExt;

mod uri;
pub use uri::Uri;

#[cfg(feature = "server")]
pub use crate::server::conn::SocketIncoming;

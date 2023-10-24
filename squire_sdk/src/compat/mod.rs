//! This module contains the compatiablity layer to abstract over if the client is running natively
//! or in WASM. The goal of this module is to eliminate the use of this outside of this module:
//! ```rust, ignore
//! #[cfg(target_family = "wasm")]
//! ```
//!
//! By no means is this an exhuastive or future-proof module. Rather, the module just implements
//! wrappers for functionalities that are presently needed.
use std::{
    fmt::Debug,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Future, FutureExt};

#[cfg(not(target_family = "wasm"))]
mod native;
#[cfg(not(target_family = "wasm"))]
pub use native::*;

#[cfg(target_family = "wasm")]
mod wasm;
#[cfg(target_family = "wasm")]
pub use wasm::*;

/// A struct that will sleep for a set amount of time. Construct by the `sleep` and `sleep_until`
/// functions.
pub struct Sleep(Pin<Box<dyn 'static + Send + Future<Output = ()>>>);

impl Debug for Sleep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sleep(..)")
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.poll_unpin(cx)
    }
}

/// A shorthand for the results of fallible Websocket operations
pub type WebsocketResult = Result<WebsocketMessage, WebsocketError>;

/// The common message return by the websocket types
#[derive(Debug)]
pub enum WebsocketMessage {
    Text(String),
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq)]
/// The common error type used by the websocket types
pub struct WebsocketError;

//! This module contains the compatiablity layer to abstract over if the client is running natively
//! or in WASM. The goal of this module is to eliminate the use of this outside of this module:
//! ```
//! #[cfg(target_family = "wasm")]
//! ```
//!
//! By no means is this an exhuastive or future-proof module. Rather, the module just implements
//! wrappers for functionalities that are presently needed.

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{future::FusedFuture, Future};

pub(crate) fn forget<T>(_: T) {}

#[cfg(not(target_family = "wasm"))]
mod native;
#[cfg(not(target_family = "wasm"))]
//pub use native::*;

//#[cfg(target_family = "wasm")]
mod wasm;
//#[cfg(target_family = "wasm")]
pub use wasm::*;

/// A common error return by the receiver half of an unbounded channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryRecvError {
    Empty,
    Disconnected,
}

impl TryRecvError {
    fn is_disconnected(&self) -> bool {
        *self == TryRecvError::Disconnected
    }
}

impl<T> Future for UnboundedReceiver<T> {
    type Output = Option<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.try_recv() {
            Ok(val) => Poll::Ready(Some(val)),
            Err(TryRecvError::Empty) => Poll::Pending,
            Err(TryRecvError::Disconnected) => Poll::Ready(None),
        }
    }
}

impl<T> FusedFuture for UnboundedReceiver<T> {
    fn is_terminated(&self) -> bool {
        self.is_disconnected()
    }
}

/// A shorthand for the results of fallible Websocket operations
pub type WebsocketResult = Result<WebsocketMessage, WebsocketError>;

/// The common message return by the websocket types
pub enum WebsocketMessage {
    Text(String),
    Bytes(Vec<u8>),
}

/// The common error type used by the websocket types
pub struct WebsocketError;

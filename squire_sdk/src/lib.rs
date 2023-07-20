//! This crate contains the model used by SquireCore's API endpoints. There are some features
//! needed only for the SquireCore server. These are enabled by default. For client usage, you can
//! disable them with `--no-default-features`.

#![warn(rust_2018_idioms)]
#![deny(
    //missing_docs,
    //missing_debug_implementations,
    rustdoc::broken_intra_doc_links,
    unreachable_pub,
    unreachable_patterns,
    unused,
    unused_results,
    unused_qualifications,
    while_true,
    trivial_casts,
    trivial_bounds,
    trivial_numeric_casts,
    unconditional_panic,
    clippy::all,
)]

use serde::{Deserialize, Serialize};
/// The module wraps and re-exports the squire_lib crate
pub use squire_lib as model;

pub mod api;
pub mod sync;
pub mod utils;

pub static COOKIE_NAME: &str = "SQUIRE_SESSION";

#[cfg(feature = "client")]
/// The default client used by non-squire_core services to communicate with squire_core
pub mod client;

#[cfg(feature = "server")]
/// The default client used by non-squire_core services to communicate with squire_core
pub mod server;

/// The errors used by this library
pub mod error;
/// The primary generic response type
pub mod response;
/// Request/response structs for SquireCore tournament apis
pub mod tournaments;
/// Request/response structs for server version
pub mod version;

/// A general-purpose enum to encode what to do with an accompanying value
#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// Add the value to a collection
    Add,
    /// Remove the value from a collection
    Delete,
}

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

/// The module wraps and re-exports the squire_lib crate
pub use squire_lib as model;

pub static COOKIE_NAME: &str = "SQUIRE_SESSION";

#[cfg(feature = "client")]
/// The default client used by non-squire_core services to communicate with squire_core
pub mod client;

#[cfg(feature = "server")]
/// The default client used by non-squire_core services to communicate with squire_core
pub mod server;

/// Contains all of the API definitions
pub mod api;
/// The primary generic response type
pub mod response;
/// Contains all of the components needed for client-server synchronization
pub mod sync;

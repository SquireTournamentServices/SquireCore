//! This crate contains the model used by SquireCore's API endpoints. There are some features
//! needed only for the SquireCore server. These are enabled by default. For client usage, you can
//! disable them with `--no-default-features`.

#![deny(
    missing_docs,
    unused_variables,
    unused_imports,
    unused_import_braces,
    rustdoc::broken_intra_doc_links,
    missing_debug_implementations,
    unreachable_pub
)]
#![warn(rust_2018_idioms)]

pub use squire_lib;

//pub mod accounts;
/// Request/response structs for SquireCore card apis
pub mod cards;
/// The errors used by this library
pub mod error;
/// Request/response structs for SquireCore tournament player apis
pub mod players;
/// The primary generic response type
pub mod response;
/// Request/response structs for SquireCore tournament apis
pub mod tournaments;
/// Request/response structs for SquireCore account apis
pub mod accounts;
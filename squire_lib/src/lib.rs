#![allow(dead_code, irrefutable_let_patterns)]
#![deny(
    //dead_code,
    unused_variables,
    unused_imports,
    unused_import_braces,
    rustdoc::broken_intra_doc_links,
    missing_debug_implementations,
    unreachable_pub,
)]
#![warn(rust_2018_idioms)]

// Used in ffi
#![cfg_attr(feature = "ffi", feature(allocator_api, slice_ptr_get))]

//#![cfg_attr(feature = "ffi", deny(improper_ctypes_definitions))]
//#![deny(improper_ctypes_definitions)]
pub mod error;
#[cfg(feature = "ffi")]
pub mod ffi;
pub mod fluid_pairings;
pub mod identifiers;
pub mod operations;
pub mod pairings;
pub mod player;
pub mod player_registry;
pub mod round;
pub mod round_registry;
pub mod scoring;
pub mod settings;
pub mod standard_scoring;
pub mod swiss_pairings;
pub mod tournament;
pub mod tournament_manager;

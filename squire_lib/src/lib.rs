//! SquireLib implements all the core tournament logic used by all Squire services. This includes
//! models for players, rounds, scoring and pairings systems, and tournaments. The client-server
//! sync protocol is also implemented here.

#![allow(dead_code, irrefutable_let_patterns)]
#![deny(
    // TODO: Un-comment after tests are written
    //dead_code,
    missing_docs,
    unused_variables,
    unused_imports,
    unused_import_braces,
    rustdoc::broken_intra_doc_links,
    missing_debug_implementations,
    unreachable_pub,
)]
#![warn(rust_2018_idioms)]
// Used in FFI for access to the allocator's api
#![cfg_attr(feature = "ffi", feature(allocator_api, slice_ptr_get))]
// TODO: Once FFI has been stablized, we should deny unsafe FFI types in FFI signatures
//#![cfg_attr(feature = "ffi", deny(improper_ctypes_definitions))]

// TODO: Once FFI is stablized, it too needs to be documented
#![cfg_attr(feature = "ffi", allow(missing_docs))]

/// Contains the errors used throughout SquireLib
pub mod error;
#[cfg(feature = "ffi")]
/// Contains the ffi C bindings used in SquireDesktop
pub mod ffi;
#[cfg(feature = "ffi")]
/// Contains the ffi C bindings for players used in SquireDesktop
pub mod ffi_player;
/// Contains the ffi C bindings for a tournament used in SquireDesktop
pub mod ffi_rounds;
#[cfg(feature = "ffi")]
/// Contains the ffi C bindings for a tournament used in SquireDesktop
pub mod ffi_tournament;

/// Contains the models for user and organization accounts
pub mod accounts;
/// Contains the models for judges and admins
pub mod admin;
/// Contains a queue-based pairings system model
pub mod fluid_pairings;
/// Contains identifiers for all major tournament types
pub mod identifiers;
/// Contains the client-server sync protocol
pub mod operations;
/// Contains model for communicating info about new pairings
pub mod pairings;
/// Contains the player model
pub mod player;
/// Contains the model that manages players
pub mod player_registry;
/// Contains the round model
pub mod round;
/// Contains the model that manages rounds
pub mod round_registry;
/// Contains the model for communicating scores
pub mod scoring;
/// Contains the models for all the different tournament settings
pub mod settings;
/// Contains the models for the standard score
pub mod standard_scoring;
/// Contains swiss pairings system model
pub mod swiss_pairings;
/// Contains the core tournament model
pub mod tournament;
/// Contains the model for the tournament manager
pub mod tournament_manager;

//! SquireLib implements all the core tournament logic used by all Squire services. This includes
//! models for players, rounds, scoring and pairings systems, and tournaments. The client-server
//! sync protocol is also implemented here.
#![warn(rust_2018_idioms)]
#![deny(
dead_code,
missing_docs,
unused_variables,
unused_imports,
unused_import_braces,
rustdoc::broken_intra_doc_links,
missing_debug_implementations,
unreachable_pub
)]

#[allow(non_camel_case_types)]
/// The numerical type used in the scoring systems
pub type r64 = num_rational::Rational32;

mod boilerplate;

/// Contains the models for user and organization accounts
pub mod accounts;
/// Contains the models for judges and admins
pub mod admin;
/// Contains the errors used throughout SquireLib
pub mod error;
/// Contains identifiers for all major tournament types
pub mod identifiers;
/// Contains the client-server sync protocol
pub mod operations;
/// Contains model for communicating info about new pairings
pub mod pairings;
/// Contains everything relating to the player model
pub mod players;
/// Contains the round model
pub mod rounds;
/// Contains the model for communicating scores
pub mod scoring;
/// Contains the models for all the different tournament settings
pub mod settings;
/// Contains the tournament model
pub mod tournament;
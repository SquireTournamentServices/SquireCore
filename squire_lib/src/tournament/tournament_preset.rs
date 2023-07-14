use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(C)]
/// An enum that encode the initial values of a tournament
pub enum TournamentPreset {
    /// The tournament will have a swiss pairing system and a standard scoring system
    Swiss,
    /// The tournament will have a fluid pairing system and a standard scoring system
    Fluid,
}

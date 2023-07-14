use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(
Serialize, Deserialize, Default, Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
)]
#[repr(C)]
/// An enum that encodes all the statuses of a tournament
pub enum TournamentStatus {
    /// The tournament can not create rounds
    #[default]
    Planned,
    /// All functionalities are unlocked
    Started,
    /// All functionalities except status changes are locked
    Frozen,
    /// The tournament is over after starting
    Ended,
    /// The tournament is over and was never started
    Cancelled,
}

impl Display for TournamentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TournamentStatus::Planned => "Planned",
                TournamentStatus::Started => "Started",
                TournamentStatus::Frozen => "Frozen",
                TournamentStatus::Ended => "Ended",
                TournamentStatus::Cancelled => "Cancelled",
            }
        )
    }
}

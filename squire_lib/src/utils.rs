use crate::{
    scoring::{ScoringSystem, StandardScoring},
    settings::{
        FluidPairingsSetting, PairingSetting, ScoringSetting, StandardScoringSetting,
        SwissPairingsSetting, TournamentSetting,
    },
};

impl From<PairingSetting> for TournamentSetting {
    fn from(other: PairingSetting) -> Self {
        TournamentSetting::PairingSetting(other)
    }
}

impl From<ScoringSetting> for TournamentSetting {
    fn from(other: ScoringSetting) -> Self {
        TournamentSetting::ScoringSetting(other)
    }
}

impl From<SwissPairingsSetting> for PairingSetting {
    fn from(other: SwissPairingsSetting) -> Self {
        PairingSetting::Swiss(other)
    }
}

impl From<FluidPairingsSetting> for PairingSetting {
    fn from(other: FluidPairingsSetting) -> Self {
        PairingSetting::Fluid(other)
    }
}

impl From<StandardScoringSetting> for ScoringSetting {
    fn from(other: StandardScoringSetting) -> Self {
        ScoringSetting::Standard(other)
    }
}

impl From<SwissPairingsSetting> for TournamentSetting {
    fn from(other: SwissPairingsSetting) -> Self {
        TournamentSetting::PairingSetting(other.into())
    }
}

impl From<FluidPairingsSetting> for TournamentSetting {
    fn from(other: FluidPairingsSetting) -> Self {
        TournamentSetting::PairingSetting(other.into())
    }
}

impl From<StandardScoringSetting> for TournamentSetting {
    fn from(other: StandardScoringSetting) -> Self {
        TournamentSetting::ScoringSetting(other.into())
    }
}

impl From<StandardScoring> for ScoringSystem {
    fn from(other: StandardScoring) -> Self {
        Self::Standard(other)
    }
}

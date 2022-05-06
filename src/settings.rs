use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TournamentSetting {
    Format(String),
    MinDeckCount(u8),
    MaxDeckCount(u8),
    RequireCheckIn(bool),
    RequireDeckReg(bool),
    PairingSetting(PairingSetting),
    ScoringSetting(ScoringSetting),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PairingSetting {
    Swiss(SwissPairingsSetting),
    Fluid(FluidPairingsSetting),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ScoringSetting {
    Standard(StandardScoringSetting),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SwissPairingsSetting {
    MatchSize(u8),
    DoCheckIns(bool),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum FluidPairingsSetting {
    MatchSize(u8),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum StandardScoringSetting {
    MatchWinPoints(f64),
    MatchDrawPoints(f64),
    MatchLossPoints(f64),
    GameWinPoints(f64),
    GameDrawPoints(f64),
    GameLossPoints(f64),
    ByePoints(f64),
    IncludeByes(bool),
    IncludeMatchPoints(bool),
    IncludeGamePoints(bool),
    IncludeMwp(bool),
    IncludeGwp(bool),
    IncludeOppMwp(bool),
    IncludeOppGwp(bool),
}

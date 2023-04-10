use std::fmt::{self, Display, Formatter};

use crate::{
    scoring::{ScoringStyle, StandardScoring},
    settings::{
        PairingSetting, PairingSettingsTree,
        ScoringSetting, ScoringSettingsTree, StandardScoringSetting, StandardScoringSettingsTree,
        TournamentSetting, TournamentSettingsTree, SwissPairingSetting, FluidPairingSetting, FluidPairingSettingsTree, GeneralSetting, SwissPairingSettingsTree, GeneralSettingsTree,
    },
};

impl From<GeneralSetting> for TournamentSetting {
    fn from(other: GeneralSetting) -> Self {
        TournamentSetting::GeneralSetting(other)
    }
}

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

impl From<SwissPairingSetting> for PairingSetting {
    fn from(other: SwissPairingSetting) -> Self {
        PairingSetting::Swiss(other)
    }
}

impl From<FluidPairingSetting> for PairingSetting {
    fn from(other: FluidPairingSetting) -> Self {
        PairingSetting::Fluid(other)
    }
}

impl From<StandardScoringSetting> for ScoringSetting {
    fn from(other: StandardScoringSetting) -> Self {
        ScoringSetting::Standard(other)
    }
}

impl From<SwissPairingSetting> for TournamentSetting {
    fn from(other: SwissPairingSetting) -> Self {
        TournamentSetting::PairingSetting(other.into())
    }
}

impl From<FluidPairingSetting> for TournamentSetting {
    fn from(other: FluidPairingSetting) -> Self {
        TournamentSetting::PairingSetting(other.into())
    }
}

impl From<StandardScoringSetting> for TournamentSetting {
    fn from(other: StandardScoringSetting) -> Self {
        TournamentSetting::ScoringSetting(other.into())
    }
}

impl From<StandardScoring> for ScoringStyle {
    fn from(other: StandardScoring) -> Self {
        Self::Standard(other)
    }
}

impl Default for TournamentSettingsTree {
    fn default() -> TournamentSettingsTree {
        TournamentSettingsTree::new()
    }
}

impl Default for GeneralSettingsTree {
    fn default() -> GeneralSettingsTree {
        GeneralSettingsTree::new()
    }
}

impl Default for PairingSettingsTree {
    fn default() -> PairingSettingsTree {
        PairingSettingsTree::new()
    }
}

impl Default for SwissPairingSettingsTree {
    fn default() -> SwissPairingSettingsTree {
        SwissPairingSettingsTree::new()
    }
}

impl Default for FluidPairingSettingsTree {
    fn default() -> FluidPairingSettingsTree {
        FluidPairingSettingsTree::new()
    }
}

impl Default for ScoringSettingsTree {
    fn default() -> ScoringSettingsTree {
        ScoringSettingsTree::new()
    }
}

impl Default for StandardScoringSettingsTree {
    fn default() -> StandardScoringSettingsTree {
        StandardScoringSettingsTree::new()
    }
}

impl Display for TournamentSetting {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use TournamentSetting::*;
        match self {
            GeneralSetting(s) => {
                write!(f, "{s}")
            }
            PairingSetting(s) => {
                write!(f, "{s}")
            }
            ScoringSetting(s) => {
                write!(f, "{s}")
            }
        }
    }
}

impl Display for GeneralSetting {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use GeneralSetting::*;
        match self {
            Format(s) => {
                write!(f, "Format: {s}")
            }
            StartingTableNumber(s) => {
                write!(f, "First Table#: {s}")
            }
            UseTableNumbers(s) => {
                write!(f, "Table#?: {s}")
            }
            MinDeckCount(s) => {
                write!(f, "Min Deck Count: {s}")
            }
            MaxDeckCount(s) => {
                write!(f, "Min Deck Count: {s}")
            }
            RequireCheckIn(s) => {
                write!(f, "Check Ins?: {}", if *s { "yes" } else { "no" })
            }
            RequireDeckReg(s) => {
                write!(f, "Deck Reg?: {}", if *s { "yes" } else { "no" })
            }
            RoundLength(dur) => {
                write!(f, "Round Length: {} sec", dur.as_secs())
            }
        }
    }
}

impl fmt::Display for PairingSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PairingSetting::*;
        match self {
            MatchSize(size) => write!(f, "Match size: {size}"),
            RepairTolerance(tol) => write!(f, "Repair tolerance: {tol}"),
            Algorithm(alg) => write!(f, "Algorithm: {alg}"),
            Swiss(s) => write!(f, "{s}"),
            Fluid(s) => write!(f, "{s}"),
        }
    }
}

impl fmt::Display for ScoringSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ScoringSetting::*;
        match self {
            Standard(s) => write!(f, "{s}"),
        }
    }
}

impl fmt::Display for SwissPairingSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SwissPairingSetting::*;
        match self {
            DoCheckIns(s) => write!(f, "Check Ins?: {s}"),
        }
    }
}

impl fmt::Display for FluidPairingSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //use FluidPairingsSetting::*;
        write!(f, "FluidPairingSetting")
    }
}

impl fmt::Display for StandardScoringSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use StandardScoringSetting::*;
        match self {
            MatchWinPoints(s) => write!(f, "Match Win: {s}"),
            MatchDrawPoints(s) => write!(f, "Match Draw: {s}"),
            MatchLossPoints(s) => write!(f, "Match Loss: {s}"),
            GameWinPoints(s) => write!(f, "Game Win: {s}"),
            GameDrawPoints(s) => write!(f, "Game Draw: {s}"),
            GameLossPoints(s) => write!(f, "Game Loss: {s}"),
            ByePoints(s) => write!(f, "Bye Win: {s}"),
            IncludeByes(s) => write!(f, "Byes?: {}", if *s { "yes" } else { "no" }),
            IncludeMatchPoints(s) => write!(f, "Match Points?: {}", if *s { "yes" } else { "no" }),
            IncludeGamePoints(s) => write!(f, "Game Points?: {}", if *s { "yes" } else { "no" }),
            IncludeMwp(s) => write!(f, "MWP?: {}", if *s { "yes" } else { "no" }),
            IncludeGwp(s) => write!(f, "GWP?: {}", if *s { "yes" } else { "no" }),
            IncludeOppMwp(s) => write!(f, "Opp MWP?: {}", if *s { "yes" } else { "no" }),
            IncludeOppGwp(s) => write!(f, "Opp GWP?: {}", if *s { "yes" } else { "no" }),
        }
    }
}

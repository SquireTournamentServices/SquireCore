use std::fmt::{self, Display, Formatter};

use crate::settings::{
    CommonPairingSetting, CommonScoringSetting, FluidPairingSetting, GeneralSetting,
    PairingSetting, PairingStyleSetting, ScoringSetting, ScoringStyleSetting,
    StandardScoringSetting, SwissPairingSetting, TournamentSetting,
};

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
            Common(s) => write!(f, "{s}"),
            Style(s) => write!(f, "{s}"),
        }
    }
}

impl fmt::Display for PairingStyleSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PairingStyleSetting::Swiss(s) => write!(f, "{s}"),
            PairingStyleSetting::Fluid(s) => write!(f, "{s}"),
        }
    }
}

impl fmt::Display for ScoringSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ScoringSetting::*;
        match self {
            Common(s) => write!(f, "{s}"),
            Style(s) => write!(f, "{s}"),
        }
    }
}

impl fmt::Display for CommonScoringSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CommonScoring Setting")
    }
}

impl fmt::Display for ScoringStyleSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ScoringStyleSetting::*;
        match self {
            Standard(s) => write!(f, "{s}"),
        }
    }
}

impl fmt::Display for CommonPairingSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CommonPairingSetting::*;
        match self {
            MatchSize(size) => write!(f, "Match Size: {size}"),
            RepairTolerance(tol) => write!(f, "Repair Tolerance: {tol}"),
            Algorithm(alg) => write!(f, "Algorithm: {alg}"),
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

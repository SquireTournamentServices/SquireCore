use core::fmt;

use serde::{Deserialize, Serialize};

use crate::tournament::TournamentPreset;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TournamentSettingsTree {
    format: String,
    starting_table_number: u64,
    use_table_numbers: bool,
    min_deck_count: u8,
    max_deck_count: u8,
    require_check_in: bool,
    require_deck_reg: bool,
    pairing_settings: PairingSettingsTree,
    scoring_settings: ScoringSettingsTree,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TournamentSetting {
    Format(String),
    StartingTableNumber(u64),
    UseTableNumbers(bool),
    MinDeckCount(u8),
    MaxDeckCount(u8),
    RequireCheckIn(bool),
    RequireDeckReg(bool),
    PairingSetting(PairingSetting),
    ScoringSetting(ScoringSetting),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PairingSettingsTree {
    swiss: SwissPairingsSettingsTree,
    fluid: FluidPairingsSettingsTree,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum PairingSetting {
    Swiss(SwissPairingsSetting),
    Fluid(FluidPairingsSetting),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ScoringSettingsTree {
    standard: StandardScoringSettingsTree,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ScoringSetting {
    Standard(StandardScoringSetting),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SwissPairingsSettingsTree {
    match_size: u8,
    do_check_ins: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum SwissPairingsSetting {
    MatchSize(u8),
    DoCheckIns(bool),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FluidPairingsSettingsTree {
    match_size: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum FluidPairingsSetting {
    MatchSize(u8),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StandardScoringSettingsTree {
    match_win_points: f64,
    match_draw_points: f64,
    match_loss_points: f64,
    game_win_points: f64,
    game_draw_points: f64,
    game_loss_points: f64,
    bye_points: f64,
    include_byes: bool,
    include_match_points: bool,
    include_game_points: bool,
    include_mwp: bool,
    include_gwp: bool,
    include_opp_mwp: bool,
    include_opp_gwp: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[repr(C)]
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

impl TournamentSettingsTree {
    pub fn as_settings(&self, preset: TournamentPreset) -> Vec<TournamentSetting> {
        use TournamentSetting::*;
        let mut digest = vec![
            Format(self.format.clone()),
            StartingTableNumber(self.starting_table_number),
            UseTableNumbers(self.use_table_numbers),
            MinDeckCount(self.min_deck_count),
            MaxDeckCount(self.max_deck_count),
            RequireCheckIn(self.require_check_in),
            RequireDeckReg(self.require_deck_reg),
        ];
        digest.extend(self.pairing_settings.as_settings(preset).into_iter());
        digest.extend(self.scoring_settings.as_settings(preset).into_iter());
        digest
    }
}

impl PairingSettingsTree {
    pub fn as_settings(&self, preset: TournamentPreset) -> Vec<TournamentSetting> {
        match preset {
            TournamentPreset::Swiss => {
                vec![
                    TournamentSetting::PairingSetting(PairingSetting::Swiss(
                        SwissPairingsSetting::MatchSize(self.swiss.match_size),
                    )),
                    TournamentSetting::PairingSetting(PairingSetting::Swiss(
                        SwissPairingsSetting::DoCheckIns(self.swiss.do_check_ins),
                    )),
                ]
            }
            TournamentPreset::Fluid => {
                vec![TournamentSetting::PairingSetting(PairingSetting::Fluid(
                    FluidPairingsSetting::MatchSize(self.fluid.match_size),
                ))]
            }
        }
    }
}

impl ScoringSettingsTree {
    pub fn as_settings(&self, _: TournamentPreset) -> Vec<TournamentSetting> {
        vec![
            TournamentSetting::ScoringSetting(ScoringSetting::Standard(
                StandardScoringSetting::MatchWinPoints(self.standard.match_win_points),
            )),
            TournamentSetting::ScoringSetting(ScoringSetting::Standard(
                StandardScoringSetting::MatchDrawPoints(self.standard.match_draw_points),
            )),
            TournamentSetting::ScoringSetting(ScoringSetting::Standard(
                StandardScoringSetting::MatchLossPoints(self.standard.match_loss_points),
            )),
            TournamentSetting::ScoringSetting(ScoringSetting::Standard(
                StandardScoringSetting::GameWinPoints(self.standard.game_win_points),
            )),
            TournamentSetting::ScoringSetting(ScoringSetting::Standard(
                StandardScoringSetting::GameDrawPoints(self.standard.game_draw_points),
            )),
            TournamentSetting::ScoringSetting(ScoringSetting::Standard(
                StandardScoringSetting::GameLossPoints(self.standard.game_loss_points),
            )),
            TournamentSetting::ScoringSetting(ScoringSetting::Standard(
                StandardScoringSetting::ByePoints(self.standard.bye_points),
            )),
            TournamentSetting::ScoringSetting(ScoringSetting::Standard(
                StandardScoringSetting::IncludeByes(self.standard.include_byes),
            )),
            TournamentSetting::ScoringSetting(ScoringSetting::Standard(
                StandardScoringSetting::IncludeMatchPoints(self.standard.include_byes),
            )),
            TournamentSetting::ScoringSetting(ScoringSetting::Standard(
                StandardScoringSetting::IncludeGamePoints(self.standard.include_byes),
            )),
            TournamentSetting::ScoringSetting(ScoringSetting::Standard(
                StandardScoringSetting::IncludeMwp(self.standard.include_mwp),
            )),
            TournamentSetting::ScoringSetting(ScoringSetting::Standard(
                StandardScoringSetting::IncludeGwp(self.standard.include_gwp),
            )),
            TournamentSetting::ScoringSetting(ScoringSetting::Standard(
                StandardScoringSetting::IncludeOppMwp(self.standard.include_opp_mwp),
            )),
            TournamentSetting::ScoringSetting(ScoringSetting::Standard(
                StandardScoringSetting::IncludeOppGwp(self.standard.include_opp_gwp),
            )),
        ]
    }
}

impl fmt::Display for TournamentSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TournamentSetting::*;
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
            PairingSetting(s) => {
                write!(f, "{s}")
            }
            ScoringSetting(s) => {
                write!(f, "{s}")
            }
        }
    }
}

impl fmt::Display for PairingSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PairingSetting::*;
        match self {
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

impl fmt::Display for SwissPairingsSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SwissPairingsSetting::*;
        match self {
            MatchSize(s) => write!(f, "Match Size: {s}"),
            DoCheckIns(s) => write!(f, "Check Ins?: {s}"),
        }
    }
}

impl fmt::Display for FluidPairingsSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use FluidPairingsSetting::*;
        match self {
            MatchSize(s) => write!(f, "Match Size: {s}"),
        }
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

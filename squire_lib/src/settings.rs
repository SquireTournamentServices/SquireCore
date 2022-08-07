use core::fmt;

use serde::{Deserialize, Serialize};

use crate::tournament::TournamentPreset;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// A set of adjustable default settings for a tournament.
pub struct TournamentSettingsTree {
    /// The default format
    pub format: String,
    /// The default starting table number
    pub starting_table_number: u64,
    /// The default table number strategy
    pub use_table_numbers: bool,
    /// The default minimum deck count
    pub min_deck_count: u8,
    /// The default maximum deck count
    pub max_deck_count: u8,
    /// The default strategy for checkins
    pub require_check_in: bool,
    /// The default strategy for deck registration
    pub require_deck_reg: bool,
    /// The default pairings settings
    pub pairing_settings: PairingSettingsTree,
    /// The default scoring settings
    pub scoring_settings: ScoringSettingsTree,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// An enum that encodes all the adjustable settings of a tournament
pub enum TournamentSetting {
    /// Adjusts the format of the tournament
    Format(String),
    /// Adjusts the starting table number of the tournament
    StartingTableNumber(u64),
    /// Adjusts if the tournament will assign table numbers
    UseTableNumbers(bool),
    /// Adjusts the minimum deck count for the tournament
    MinDeckCount(u8),
    /// Adjusts the maximum deck count for the tournament
    MaxDeckCount(u8),
    /// Adjusts if the tournament requires checkins
    RequireCheckIn(bool),
    /// Adjusts if the tournament requires deck registration
    RequireDeckReg(bool),
    /// Adjusts a pairing system setting
    PairingSetting(PairingSetting),
    /// Adjusts a scoring system setting
    ScoringSetting(ScoringSetting),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// A set of adjustable default settings for a pairings system.
pub struct PairingSettingsTree {
    /// Settigns for swiss pairings
    pub swiss: SwissPairingsSettingsTree,
    /// Settigns for fluid pairings
    pub fluid: FluidPairingsSettingsTree,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// An enum that encodes all the adjustable settings of all pairing systems
pub enum PairingSetting {
    /// Settings for the swiss pairings system
    Swiss(SwissPairingsSetting),
    /// Settings for the fluid pairings system
    Fluid(FluidPairingsSetting),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// A set of adjustable default settings for a scoring system.
pub struct ScoringSettingsTree {
    /// The settings tree for standard scoring
    pub standard: StandardScoringSettingsTree,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// An enum that encodes all the adjustable settings of all scoring systems
pub enum ScoringSetting {
    /// Settings for the standard scoring system
    Standard(StandardScoringSetting),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// A set of adjustable default settings for a swiss pairing system.
pub struct SwissPairingsSettingsTree {
    /// The default round size
    pub match_size: u8,
    /// The default on the check in strategy
    pub do_check_ins: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[repr(C)]
/// An enum that encodes all the adjustable settings of swiss pairing systems
pub enum SwissPairingsSetting {
    /// The number of players per round
    MatchSize(u8),
    /// Whether or not player need to check in before a round is paired
    DoCheckIns(bool),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// A set of adjustable default settings for a fluid pairing system.
pub struct FluidPairingsSettingsTree {
    /// The default round size
    pub match_size: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[repr(C)]
/// An enum that encodes all the adjustable settings of fluid pairing systems
pub enum FluidPairingsSetting {
    /// The number of players per round
    MatchSize(u8),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// A set of adjustable default settings for a standard scoring system.
pub struct StandardScoringSettingsTree {
    /// The default number of points a match win is worth
    pub match_win_points: f64,
    /// The default number of points a match draw is worth
    pub match_draw_points: f64,
    /// The default number of points a match loss is worth
    pub match_loss_points: f64,
    /// The default number of points a game win is worth
    pub game_win_points: f64,
    /// The default number of points a game draw is worth
    pub game_draw_points: f64,
    /// The default number of points a game loss is worth
    pub game_loss_points: f64,
    /// The default number of points a bye is worth
    pub bye_points: f64,
    /// The default on if byes are used in scoring
    pub include_byes: bool,
    /// The default on if match points are used in scoring
    pub include_match_points: bool,
    /// The default on if game points are used in scoring
    pub include_game_points: bool,
    /// The default on if mwp is used in scoring
    pub include_mwp: bool,
    /// The default on if gwp is used in scoring
    pub include_gwp: bool,
    /// The default on if opponent mwp is used in scoring
    pub include_opp_mwp: bool,
    /// The default on if opponent gwp is used in scoring
    pub include_opp_gwp: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[repr(C)]
/// An enum that encodes all the adjustable settings of standard scoring systems
pub enum StandardScoringSetting {
    /// Adjusts the number of points a match win is worth
    MatchWinPoints(f64),
    /// Adjusts the number of points a match draw is worth
    MatchDrawPoints(f64),
    /// Adjusts the number of points a match loss is worth
    MatchLossPoints(f64),
    /// Adjusts the number of points a game win is worth
    GameWinPoints(f64),
    /// Adjusts the number of points a game draw is worth
    GameDrawPoints(f64),
    /// Adjusts the number of points a game loss is worth
    GameLossPoints(f64),
    /// Adjusts the number of points a bye is worth
    ByePoints(f64),
    /// Adjusts if byes are used in scoring
    IncludeByes(bool),
    /// Adjusts if match points are used in scoring
    IncludeMatchPoints(bool),
    /// Adjusts if game points are used in scoring
    IncludeGamePoints(bool),
    /// Adjusts if mwp is used in scoring
    IncludeMwp(bool),
    /// Adjusts if gwp is used in scoring
    IncludeGwp(bool),
    /// Adjusts if opponent mwp is used in scoring
    IncludeOppMwp(bool),
    /// Adjusts if opponent gwp is used in scoring
    IncludeOppGwp(bool),
}

impl TournamentSettingsTree {
    /// Converts the settings tree into a vector of tournaments settings, the preset determines
    /// which system subtree is used
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
    /// Converts pairings setting tree into a vector of tournament settings
    pub fn as_settings(&self, preset: TournamentPreset) -> Vec<TournamentSetting> {
        match preset {
            TournamentPreset::Swiss => {
                vec![
                    SwissPairingsSetting::MatchSize(self.swiss.match_size).into(),
                    SwissPairingsSetting::DoCheckIns(self.swiss.do_check_ins).into(),
                ]
            }
            TournamentPreset::Fluid => {
                vec![FluidPairingsSetting::MatchSize(self.fluid.match_size).into()]
            }
        }
    }
}

impl ScoringSettingsTree {
    /// Converts the tree in a vector of tournament settings, one for every field
    pub fn as_settings(&self, _: TournamentPreset) -> Vec<TournamentSetting> {
        use StandardScoringSetting::*;
        vec![
            MatchWinPoints(self.standard.match_win_points).into(),
            MatchDrawPoints(self.standard.match_draw_points).into(),
            MatchLossPoints(self.standard.match_loss_points).into(),
            GameWinPoints(self.standard.game_win_points).into(),
            GameDrawPoints(self.standard.game_draw_points).into(),
            GameLossPoints(self.standard.game_loss_points).into(),
            ByePoints(self.standard.bye_points).into(),
            IncludeByes(self.standard.include_byes).into(),
            IncludeMatchPoints(self.standard.include_byes).into(),
            IncludeGamePoints(self.standard.include_byes).into(),
            IncludeMwp(self.standard.include_mwp).into(),
            IncludeGwp(self.standard.include_gwp).into(),
            IncludeOppMwp(self.standard.include_opp_mwp).into(),
            IncludeOppGwp(self.standard.include_opp_gwp).into(),
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

impl TournamentSettingsTree {
    /// Creates a settings tree for all tournament settings
    pub fn new() -> TournamentSettingsTree {
        TournamentSettingsTree {
            format: "Pioneer".into(),
            starting_table_number: 1,
            use_table_numbers: true,
            min_deck_count: 0,
            max_deck_count: 1,
            require_check_in: false,
            require_deck_reg: false,
            pairing_settings: PairingSettingsTree::new(),
            scoring_settings: ScoringSettingsTree::new(),
        }
    }
}

impl PairingSettingsTree {
    /// Creates a settings tree for all pairings systems
    pub fn new() -> PairingSettingsTree {
        PairingSettingsTree {
            swiss: SwissPairingsSettingsTree::new(),
            fluid: FluidPairingsSettingsTree::new(),
        }
    }
}

impl SwissPairingsSettingsTree {
    /// Creates a settings tree for fluid pairing systems
    pub fn new() -> SwissPairingsSettingsTree {
        SwissPairingsSettingsTree {
            match_size: 2,
            do_check_ins: false,
        }
    }
}

impl FluidPairingsSettingsTree {
    /// Creates a settings tree for fluid pairing systems
    pub fn new() -> FluidPairingsSettingsTree {
        FluidPairingsSettingsTree {
            match_size: 2,
        }
    }
}

impl ScoringSettingsTree {
    /// Creates a settings tree for all scoring systems
    pub fn new() -> ScoringSettingsTree {
        ScoringSettingsTree {
            standard: StandardScoringSettingsTree::new(),
        }
    }
}

impl StandardScoringSettingsTree {
    /// Creates a settings tree for standard scoring systems
    pub fn new() -> StandardScoringSettingsTree {
        StandardScoringSettingsTree {
        match_win_points: 3.0,
        match_draw_points: 1.0,
        match_loss_points: 0.0,
        game_win_points: 3.0,
        game_draw_points: 1.0,
        game_loss_points: 0.0,
        bye_points: 3.0,
        include_byes: true,
        include_match_points: true,
        include_game_points: true,
        include_mwp: true,
        include_gwp: true,
        include_opp_mwp: true,
        include_opp_gwp: true,
        }
    }
}

impl Default for TournamentSettingsTree {
    fn default() -> TournamentSettingsTree {
        TournamentSettingsTree::new()
    }
}

impl Default for PairingSettingsTree {
    fn default() -> PairingSettingsTree {
        PairingSettingsTree::new()
    }
}

impl Default for SwissPairingsSettingsTree {
    fn default() -> SwissPairingsSettingsTree {
        SwissPairingsSettingsTree::new()
    }
}

impl Default for FluidPairingsSettingsTree {
    fn default() -> FluidPairingsSettingsTree {
        FluidPairingsSettingsTree::new()
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

impl Into<TournamentSetting> for PairingSetting {
    fn into(self) -> TournamentSetting {
        TournamentSetting::PairingSetting(self)
    }
}

impl Into<TournamentSetting> for ScoringSetting {
    fn into(self) -> TournamentSetting {
        TournamentSetting::ScoringSetting(self)
    }
}

impl Into<PairingSetting> for SwissPairingsSetting {
    fn into(self) -> PairingSetting {
        PairingSetting::Swiss(self)
    }
}

impl Into<PairingSetting> for FluidPairingsSetting {
    fn into(self) -> PairingSetting {
        PairingSetting::Fluid(self)
    }
}

impl Into<ScoringSetting> for StandardScoringSetting {
    fn into(self) -> ScoringSetting {
        ScoringSetting::Standard(self)
    }
}

impl Into<TournamentSetting> for SwissPairingsSetting {
    fn into(self) -> TournamentSetting {
        TournamentSetting::PairingSetting(self.into())
    }
}

impl Into<TournamentSetting> for FluidPairingsSetting {
    fn into(self) -> TournamentSetting {
        TournamentSetting::PairingSetting(self.into())
    }
}

impl Into<TournamentSetting> for StandardScoringSetting {
    fn into(self) -> TournamentSetting {
        TournamentSetting::ScoringSetting(self.into())
    }
}

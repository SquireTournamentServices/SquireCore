use serde::{Deserialize, Serialize};

use crate::{
    operations::{OpData, OpResult},
    r64,
    tournament::tournament::TournamentPreset,
};

/// An enum that encodes all the adjustable settings of all scoring systems
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub enum ScoringSetting {
    /// Settings common to all scoring systems
    Common(CommonScoringSetting),
    /// Settings for the scoring style
    Style(ScoringStyleSetting),
}

/// Settings for a given scoring style
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub enum ScoringStyleSetting {
    /// Settings for the standard scoring style
    Standard(StandardScoringSetting),
}

/// An enum that captures common settings of all scoring systems
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub enum CommonScoringSetting {}

/// The set of settings common to all scoring systems
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct CommonScoringSettingsTree;

/// A enum that holds settings for the active scoring sytle
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub enum ScoringStyleSettingsTree {
    /// The set of settings for standard-style scoring
    Standard(StandardScoringSettingsTree),
}

/// A structure that holds a value for each scoring setting
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct ScoringSettingsTree {
    /// Settings used by all scoring methods
    #[serde(default)]
    pub common: CommonScoringSettingsTree,
    /// The settings for the style of scoring being used
    pub style: ScoringStyleSettingsTree,
}

impl ScoringSettingsTree {
    /// Creates a new, default settings tree
    pub fn new(preset: TournamentPreset) -> Self {
        Self {
            common: CommonScoringSettingsTree,
            style: ScoringStyleSettingsTree::new(preset),
        }
    }

    /// Creates a new settings tree with the given format field
    pub fn update(&mut self, setting: ScoringSetting) -> OpResult {
        match setting {
            ScoringSetting::Common(setting) => self.common.update(setting),
            ScoringSetting::Style(setting) => self.style.update(setting),
        }
    }

    /// Returns an iterator over all the contained settings
    pub fn iter(&self) -> impl Iterator<Item=ScoringSetting> {
        self.common
            .iter()
            .map(Into::into)
            .chain(self.style.iter().map(Into::into))
    }
}

impl CommonScoringSettingsTree {
    /// Creates a new, default settings tree
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new settings tree with the given format field
    pub fn update(&mut self, setting: CommonScoringSetting) -> ! {
        match setting {}
    }

    /// Returns an iterator over all the contained settings
    pub fn iter(&self) -> impl Iterator<Item=CommonScoringSetting> {
        Option::<CommonScoringSetting>::None
            .into_iter()
            .map(Into::into)
    }
}

impl ScoringStyleSettingsTree {
    /// Creates a new, default settings tree
    pub fn new(_: TournamentPreset) -> Self {
        Self::Standard(Default::default())
    }

    /// Creates a new settings tree with the given format field
    pub fn update(&mut self, setting: ScoringStyleSetting) -> OpResult {
        match (self, setting) {
            (ScoringStyleSettingsTree::Standard(style), ScoringStyleSetting::Standard(setting)) => {
                style.update(setting)
            }
        }
        Ok(OpData::Nothing)
    }

    /// Returns an iterator over all the contained settings
    pub fn iter(&self) -> impl Iterator<Item=ScoringStyleSetting> {
        match self {
            ScoringStyleSettingsTree::Standard(tree) => tree.iter().map(Into::into),
        }
    }
}

/// An enum that encodes all the adjustable settings of standard scoring systems
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum StandardScoringSetting {
    /// Adjusts the number of points a match win is worth
    MatchWinPoints(r64),
    /// Adjusts the number of points a match draw is worth
    MatchDrawPoints(r64),
    /// Adjusts the number of points a match loss is worth
    MatchLossPoints(r64),
    /// Adjusts the number of points a game win is worth
    GameWinPoints(r64),
    /// Adjusts the number of points a game draw is worth
    GameDrawPoints(r64),
    /// Adjusts the number of points a game loss is worth
    GameLossPoints(r64),
    /// Adjusts the number of points a bye is worth
    ByePoints(r64),
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

/// A structure that holds a value for each scoring setting
#[allow(missing_docs)]
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct StandardScoringSettingsTree {
    pub match_win_points: r64,
    pub match_draw_points: r64,
    pub match_loss_points: r64,
    pub game_win_points: r64,
    pub game_draw_points: r64,
    pub game_loss_points: r64,
    pub bye_points: r64,
    pub include_byes: bool,
    pub include_match_points: bool,
    pub include_game_points: bool,
    pub include_mwp: bool,
    pub include_gwp: bool,
    pub include_opp_mwp: bool,
    pub include_opp_gwp: bool,
}

impl StandardScoringSettingsTree {
    /// Creates a new, default settings tree
    pub fn new() -> Self {
        Self {
            match_win_points: r64::from_integer(3),
            match_draw_points: r64::from_integer(1),
            match_loss_points: r64::from_integer(0),
            game_win_points: r64::from_integer(3),
            game_draw_points: r64::from_integer(1),
            game_loss_points: r64::from_integer(0),
            bye_points: r64::from_integer(3),
            include_byes: true,
            include_match_points: true,
            include_game_points: true,
            include_mwp: true,
            include_gwp: true,
            include_opp_mwp: true,
            include_opp_gwp: true,
        }
    }

    /// Creates a new settings tree with the given format field
    pub fn update(&mut self, setting: StandardScoringSetting) {
        match setting {
            StandardScoringSetting::MatchWinPoints(points) => self.match_win_points = points,
            StandardScoringSetting::MatchDrawPoints(points) => self.match_draw_points = points,
            StandardScoringSetting::MatchLossPoints(points) => self.match_loss_points = points,
            StandardScoringSetting::GameWinPoints(points) => self.game_win_points = points,
            StandardScoringSetting::GameDrawPoints(points) => self.game_draw_points = points,
            StandardScoringSetting::GameLossPoints(points) => self.game_loss_points = points,
            StandardScoringSetting::ByePoints(points) => self.bye_points = points,
            StandardScoringSetting::IncludeByes(include) => self.include_byes = include,
            StandardScoringSetting::IncludeMatchPoints(include) => {
                self.include_match_points = include
            }
            StandardScoringSetting::IncludeGamePoints(include) => {
                self.include_game_points = include
            }
            StandardScoringSetting::IncludeMwp(include) => self.include_mwp = include,
            StandardScoringSetting::IncludeGwp(include) => self.include_gwp = include,
            StandardScoringSetting::IncludeOppMwp(include) => self.include_opp_mwp = include,
            StandardScoringSetting::IncludeOppGwp(include) => self.include_opp_gwp = include,
        }
    }

    /// Returns an iterator over all the contained settings
    pub fn iter(&self) -> impl Iterator<Item=StandardScoringSetting> {
        vec![
            StandardScoringSetting::MatchWinPoints(self.match_win_points),
            StandardScoringSetting::MatchDrawPoints(self.match_draw_points),
            StandardScoringSetting::MatchLossPoints(self.match_loss_points),
            StandardScoringSetting::GameWinPoints(self.game_win_points),
            StandardScoringSetting::GameDrawPoints(self.game_draw_points),
            StandardScoringSetting::GameLossPoints(self.game_loss_points),
            StandardScoringSetting::ByePoints(self.bye_points),
            StandardScoringSetting::IncludeByes(self.include_byes),
            StandardScoringSetting::IncludeMatchPoints(self.include_match_points),
            StandardScoringSetting::IncludeGamePoints(self.include_game_points),
            StandardScoringSetting::IncludeMwp(self.include_mwp),
            StandardScoringSetting::IncludeGwp(self.include_gwp),
            StandardScoringSetting::IncludeOppMwp(self.include_opp_mwp),
            StandardScoringSetting::IncludeOppGwp(self.include_opp_gwp),
        ]
            .into_iter()
    }
}

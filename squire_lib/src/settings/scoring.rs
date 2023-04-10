use serde::{Serialize, Deserialize};

use crate::r64;


/// An enum that encodes all the adjustable settings of all scoring systems
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub enum ScoringSetting {
    /// Settings for the standard scoring system
    Standard(StandardScoringSetting),
}

/// A structure that holds a value for each scoring setting
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct ScoringSettingsTree {
    standard: StandardScoringSettingsTree,
}

impl ScoringSettingsTree {
    /// Creates a new, default settings tree
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new settings tree with the given format field
    pub fn update(&mut self, setting: ScoringSetting) {
        #[allow(irrefutable_let_patterns)]
        match setting {
            ScoringSetting::Standard(setting) => self.standard.update(setting),
        }
    }

    /// Returns an iterator over all the contained settings
    pub fn iter(&self) -> impl Iterator<Item = ScoringSetting> {
        self.standard.iter().map(Into::into)
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
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct StandardScoringSettingsTree {
    match_win_points: r64,
    match_draw_points: r64,
    match_loss_points: r64,
    game_win_points: r64,
    game_draw_points: r64,
    game_loss_points: r64,
    bye_points: r64,
    include_byes: bool,
    include_match_points: bool,
    include_game_points: bool,
    include_mwp: bool,
    include_gwp: bool,
    include_opp_mwp: bool,
    include_opp_gwp: bool,
}

impl StandardScoringSettingsTree {
    /// Creates a new, default settings tree
    pub fn new() -> Self {
        Self::default()
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
            StandardScoringSetting::IncludeMatchPoints(include) => self.include_match_points = include,
            StandardScoringSetting::IncludeGamePoints(include) => self.include_game_points = include,
            StandardScoringSetting::IncludeMwp(include) => self.include_mwp = include,
            StandardScoringSetting::IncludeGwp(include) => self.include_gwp = include,
            StandardScoringSetting::IncludeOppMwp(include) => self.include_opp_mwp = include,
            StandardScoringSetting::IncludeOppGwp(include) => self.include_opp_gwp = include,
        }
    }

    /// Returns an iterator over all the contained settings
    pub fn iter(&self) -> impl Iterator<Item = StandardScoringSetting> {
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
        ].into_iter()
    }
}

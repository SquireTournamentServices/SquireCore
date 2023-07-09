use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::SettingsTree;
use crate::{
    error::TournamentError,
    operations::{OpData, OpResult},
};

/// An enum that encode all of the general tournament settings
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub enum GeneralSetting {
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
    /// Adjusts the amount of time new rounds will have
    RoundLength(Duration),
}

/// A structure that holds a value for each general tournament setting
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct GeneralSettingsTree {
    /// The format that is being played at the tournament
    pub format: String,
    /// The first table number to use when accessing table numbers
    pub starting_table_number: u64,
    /// Whether or not to use table numbers
    pub use_table_number: bool,
    /// The minimum number of decks that a player needs to have for the tournament
    pub min_deck_count: u8,
    /// The maximum number of decks that a player can have at a time
    pub max_deck_count: u8,
    /// Whether or not players must check into the tournament
    pub require_check_in: bool,
    /// Whether or not players must submit at least the minimum number of decks
    pub require_deck_reg: bool,
    /// The length of all new rounds
    pub round_length: Duration,
}

impl GeneralSettingsTree {
    /// Creates a new settings tree with the given format field
    pub fn with_format(format: String) -> Self {
        let mut digest = Self::new();
        digest.format = format;
        digest
    }
}

impl SettingsTree for GeneralSettingsTree {
    type Setting = GeneralSetting;

    fn update(&mut self, setting: Self::Setting) -> OpResult {
        match setting {
            GeneralSetting::Format(format) => self.format = format,
            GeneralSetting::StartingTableNumber(num) => self.starting_table_number = num,
            GeneralSetting::UseTableNumbers(num) => self.use_table_number = num,
            GeneralSetting::MinDeckCount(count) if count <= self.max_deck_count => {
                self.min_deck_count = count
            }
            GeneralSetting::MinDeckCount(_) => return Err(TournamentError::InvalidDeckCount),
            GeneralSetting::MaxDeckCount(count) if count >= self.min_deck_count => {
                self.max_deck_count = count
            }
            GeneralSetting::MaxDeckCount(_) => return Err(TournamentError::InvalidDeckCount),
            GeneralSetting::RequireCheckIn(check_in) => self.require_check_in = check_in,
            GeneralSetting::RequireDeckReg(deck_reg) => self.require_deck_reg = deck_reg,
            GeneralSetting::RoundLength(len) => self.round_length = len,
        }
        Ok(OpData::Nothing)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Self::Setting>> {
        Box::new(
            [
                GeneralSetting::Format(self.format.clone()),
                GeneralSetting::StartingTableNumber(self.starting_table_number),
                GeneralSetting::UseTableNumbers(self.use_table_number),
                GeneralSetting::MinDeckCount(self.min_deck_count),
                GeneralSetting::MaxDeckCount(self.max_deck_count),
                GeneralSetting::RequireCheckIn(self.require_check_in),
                GeneralSetting::RequireDeckReg(self.require_deck_reg),
                GeneralSetting::RoundLength(self.round_length),
            ]
            .into_iter(),
        )
    }
}

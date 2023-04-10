use std::time::Duration;

use serde::{Deserialize, Serialize};

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
    pub(crate) format: String,
    pub(crate) starting_table_number: u64,
    pub(crate) use_table_number: bool,
    pub(crate) min_deck_count: u8,
    pub(crate) max_deck_count: u8,
    pub(crate) require_check_in: bool,
    pub(crate) require_deck_reg: bool,
    pub(crate) round_length: Duration,
}

impl GeneralSettingsTree {
    /// Creates a new, default settings tree
    pub fn new() -> Self {
        Self {
            format: "Pioneer".into(),
            starting_table_number: 1,
            use_table_number: true,
            min_deck_count: 0,
            max_deck_count: 1,
            require_check_in: false,
            require_deck_reg: false,
            round_length: Duration::from_secs(3000),
        }
    }
    
    /// Creates a new settings tree with the given format field
    pub fn with_format(format: String) -> Self {
        let mut digest = Self::new();
        digest.format = format;
        digest
    }

    /// Updates the settings tree, replacing one setting with the given setting
    pub fn update(&mut self, setting: GeneralSetting) {
        match setting {
            GeneralSetting::Format(format) => self.format = format,
            GeneralSetting::StartingTableNumber(num) => self.starting_table_number = num,
            GeneralSetting::UseTableNumbers(num) => self.use_table_number = num,
            GeneralSetting::MinDeckCount(count) => self.min_deck_count = count,
            GeneralSetting::MaxDeckCount(count) => self.max_deck_count = count,
            GeneralSetting::RequireCheckIn(check_in) => self.require_check_in = check_in,
            GeneralSetting::RequireDeckReg(deck_reg) => self.require_deck_reg = deck_reg,
            GeneralSetting::RoundLength(len) => self.round_length = len,
        }
    }

    /// Returns an iterator over all the contained settings
    pub fn iter(&self) -> impl Iterator<Item = GeneralSetting> {
        vec![
            GeneralSetting::Format(self.format.clone()),
            GeneralSetting::StartingTableNumber(self.starting_table_number),
            GeneralSetting::UseTableNumbers(self.use_table_number),
            GeneralSetting::MinDeckCount(self.min_deck_count),
            GeneralSetting::MaxDeckCount(self.max_deck_count),
            GeneralSetting::RequireCheckIn(self.require_check_in),
            GeneralSetting::RequireDeckReg(self.require_deck_reg),
            GeneralSetting::RoundLength(self.round_length),
        ].into_iter()
    }
}

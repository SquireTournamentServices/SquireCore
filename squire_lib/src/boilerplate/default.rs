//! Implementations of defualt values for various types

#![allow(clippy::derivable_impls)]

use std::time::Duration;

use crate::{
    pairings::PairingAlgorithm,
    r64,
    settings::{
        CommonScoringSettingsTree, FluidPairingSettingsTree, GeneralSettingsTree,
        PairingCommonSettingsTree, PairingSettingsTree, PairingStyleSettingsTree,
        ScoringStyleSettingsTree, StandardScoringSettingsTree, SwissPairingSettingsTree,
    },
    tournament::TournamentPreset,
};

/* --------- Defaults for settings trees --------- */

impl Default for GeneralSettingsTree {
    fn default() -> Self {
        Self {
            format: "Pioneer".to_owned(),
            starting_table_number: 1,
            use_table_number: true,
            min_deck_count: 0,
            max_deck_count: 1,
            require_check_in: false,
            require_deck_reg: false,
            round_length: Duration::from_secs(3000),
        }
    }
}

impl Default for PairingCommonSettingsTree {
    fn default() -> Self {
        Self {
            match_size: 2,
            repair_tolerance: 0,
            algorithm: PairingAlgorithm::Branching,
        }
    }
}

impl Default for SwissPairingSettingsTree {
    fn default() -> Self {
        Self { do_checkins: false }
    }
}

impl Default for FluidPairingSettingsTree {
    fn default() -> Self {
        Self {}
    }
}

impl Default for CommonScoringSettingsTree {
    fn default() -> Self {
        Self {}
    }
}

impl Default for ScoringStyleSettingsTree {
    fn default() -> Self {
        Self::Standard(Default::default())
    }
}

impl Default for StandardScoringSettingsTree {
    fn default() -> Self {
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
}

impl Default for PairingSettingsTree {
    fn default() -> Self {
        Self::with_preset(TournamentPreset::Swiss)
    }
}

impl Default for PairingStyleSettingsTree {
    fn default() -> Self {
        Self::Swiss(Default::default())
    }
}

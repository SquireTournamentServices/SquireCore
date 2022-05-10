use crate::{
    error::TournamentError,
    fluid_pairings::FluidPairings,
    operations::TournOp,
    player::{Player, PlayerId},
    player_registry::{PlayerIdentifier, PlayerRegistry},
    round::{Round, RoundId, RoundResult, RoundStatus},
    round_registry::{RoundIdentifier, RoundRegistry},
    scoring::{Score, Standings},
    settings::{
        self, FluidPairingsSetting, PairingSetting, ScoringSetting, StandardScoringSetting,
        SwissPairingsSetting, TournamentSetting,
    },
    standard_scoring::{StandardScore, StandardScoring},
    swiss_pairings::SwissPairings,
    tournament::{Tournament, TournamentPreset},
};

use mtgjson::model::deck::Deck;

use libc::c_char;
use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    hash::{Hash, Hasher},
    time::Duration,
};

impl Tournament {
    /// Returns 0 if everything is ok.
    /// Returns 1 if there is an error with the name conversion
    /// Returns 2 if there is an error with the format conversion
    #[allow(unused_assignments)]
    #[no_mangle]
    pub extern "C" fn from_preset_c(
        mut expected: *mut Self,
        name_buf: *mut c_char,
        preset: TournamentPreset,
        format_buf: *mut c_char,
    ) -> usize {
        let name_str = unsafe { CStr::from_ptr(name_buf) };
        let name = match name_str.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                return 1;
            }
        };
        let format_str = unsafe { CStr::from_ptr(format_buf) };
        let format = match format_str.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                return 2;
            }
        };
        let tourn = Box::new(Tournament::from_preset(name, preset, format));
        expected = Box::into_raw(tourn);
        0
    }

    /// Returns 0 if everything is ok.
    /// Returns 1 if there is a TournamentError::IncorrectStatus,
    /// Returns 2 if there is a TournamentError::PlayerLookup,
    /// Returns 3 if there is a TournamentError::RoundLookup,
    /// Returns 4 if there is a TournamentError::DeckLookup,
    /// Returns 5 if there is a TournamentError::RegClosed,
    /// Returns 6 if there is a TournamentError::PlayerNotInRound,
    /// Returns 7 if there is a TournamentError::NoActiveRound,
    /// Returns 8 if there is a TournamentError::InvalidBye,
    /// Returns 9 if there is a TournamentError::ActiveMatches,
    /// Returns 10 if there is a TournamentError::PlayerNotCheckedIn,
    /// Returns 11 if there is a TournamentError::IncompatiblePairingSystem,
    /// Returns 12 if there is a TournamentError::IncompatibleScoringSystem,
    #[no_mangle]
    #[allow(improper_ctypes_definitions)]
    pub extern "C" fn apply_op_c(&mut self, op: TournOp) -> usize {
        use TournamentError::*;
        match self.apply_op(op) {
            Ok(_) => 0,
            Err(e) => match e {
                IncorrectStatus => 1,
                PlayerLookup => 2,
                RoundLookup => 3,
                DeckLookup => 4,
                RegClosed => 5,
                PlayerNotInRound => 6,
                NoActiveRound => 7,
                InvalidBye => 8,
                ActiveMatches => 9,
                PlayerNotCheckedIn => 10,
                IncompatiblePairingSystem => 11,
                IncompatibleScoringSystem => 12,
            },
        }
    }

    /// Returns 0 if everything is ok.
    /// Returns 1 if the player could not be found
    #[allow(unused_assignments)]
    #[no_mangle]
    pub extern "C" fn get_player_c(
        &self,
        mut expected: *const Player,
        ident: &PlayerIdentifier,
    ) -> usize {
        match self.get_player(ident) {
            Ok(p) => {
                expected = &p;
                0
            }
            Err(_) => 1,
        }
    }

    /// Returns 0 if everything is ok.
    /// Returns 1 if the round could not be found
    #[allow(unused_assignments)]
    #[no_mangle]
    pub extern "C" fn get_round_c(
        &self,
        mut expected: *const Round,
        ident: &RoundIdentifier,
    ) -> usize {
        match self.get_round(ident) {
            Ok(m) => {
                expected = &m;
                0
            }
            Err(_) => 1,
        }
    }

    /// Returns 0 if everything is ok.
    /// Returns 1 if there is a string conversion error.
    /// Returns 2 if the player could not be found.
    /// Returns 3 if the player could be found, but a deck by the give name can't be
    #[no_mangle]
    pub fn get_player_deck_c(
        &self,
        expected: *const Deck,
        ident: &PlayerIdentifier,
        name: CString,
    ) -> usize {
        use TournamentError::*;
        let rust_name = match name.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                return 1;
            }
        };
        match self.get_player_deck(ident, rust_name) {
            Ok(deck) => {
                let expected = deck;
                0
            }
            Err(e) => match e {
                PlayerLookup => 2,
                DeckLookup => 3,
                _ => {
                    unreachable!("The get_player_deck method will only ever return Player or Deck lookup error.")
                }
            },
        }
    }

    /// Returns `0` if the player could be found and `1` if they could be not be found.
    #[allow(unused_assignments)]
    #[no_mangle]
    pub extern "C" fn get_player_round_c(
        &self,
        mut expected: *const RoundId,
        ident: &PlayerIdentifier,
    ) -> usize {
        match self.get_player_round(ident) {
            Ok(r) => {
                expected = &r;
                0
            }
            Err(_) => 1,
        }
    }

    #[no_mangle]
    pub extern "C" fn get_standings_c(&self) -> Standings<StandardScore> {
        self.get_standings()
    }

    #[no_mangle]
    pub extern "C" fn is_planned_c(&self) -> bool {
        self.is_planned()
    }

    #[no_mangle]
    pub extern "C" fn is_frozen_c(&self) -> bool {
        self.is_frozen()
    }

    #[no_mangle]
    pub extern "C" fn is_active_c(&self) -> bool {
        self.is_active()
    }

    #[no_mangle]
    pub extern "C" fn is_dead_c(&self) -> bool {
        self.is_dead()
    }
}

impl TournOp {
    #[allow(unused_assignments)]
    pub extern "C" fn new_update_reg_op_c(mut expected: *const TournOp, status: bool) {
        let alloc = Box::new(Self::UpdateReg(status));
        expected = Box::into_raw(alloc);
    }
    
    #[allow(unused_assignments)]
    pub extern "C" fn new_start_op_c(mut expected: *const TournOp) {
        let alloc = Box::new(Self::Start());
        expected = Box::into_raw(alloc);
    }
    
    #[allow(unused_assignments)]
    pub extern "C" fn new_freeze_op_c(mut expected: *const TournOp) {
        let alloc = Box::new(Self::Freeze());
        expected = Box::into_raw(alloc);
    }
    
    #[allow(unused_assignments)]
    pub extern "C" fn new_thaw_op_c(mut expected: *const TournOp) {
        let alloc = Box::new(Self::Thaw());
        expected = Box::into_raw(alloc);
    }
    
    #[allow(unused_assignments)]
    pub extern "C" fn new_end_op_c(mut expected: *const TournOp) {
        let alloc = Box::new(Self::End());
        expected = Box::into_raw(alloc);
    }
    
    #[allow(unused_assignments)]
    pub extern "C" fn new_cancel_op_c(mut expected: *const TournOp) {
        let alloc = Box::new(Self::Cancel());
        expected = Box::into_raw(alloc);
    }
    
    #[allow(unused_assignments)]
    pub extern "C" fn new_check_in_op_c(mut expected: *const TournOp, plyr: PlayerId) {
        let alloc = Box::new(Self::CheckIn(PlayerIdentifier::Id(plyr)));
        expected = Box::into_raw(alloc);
    }
    
    /// Returns 0 if everything is ok.
    /// Returns 1 if there is an issue with name conversion
    #[allow(unused_assignments)]
    pub extern "C" fn new_register_player_op_c(mut expected: *const TournOp, name_buf: *mut c_char) -> usize {
        let name_str = unsafe { CStr::from_ptr(name_buf) };
        let name = match name_str.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                return 1;
            }
        };
        let alloc = Box::new(Self::RegisterPlayer(name));
        expected = Box::into_raw(alloc);
        0
    }
    
    #[allow(unused_assignments)]
    pub extern "C" fn new_record_result_op_c(
        mut expected: *const TournOp,
        rnd: RoundId,
        result: RoundResult,
    ) {
        let alloc = Box::new(Self::RecordResult(RoundIdentifier::Id(rnd), result));
        expected = Box::into_raw(alloc);
    }
    
    #[allow(unused_assignments)]
    pub extern "C" fn new_confirm_result_op_c(mut expected: *const TournOp, plyr: PlayerId) {
        let alloc = Box::new(Self::ConfirmResult(PlayerIdentifier::Id(plyr)));
        expected = Box::into_raw(alloc);
    }
    
    #[allow(unused_assignments)]
    pub extern "C" fn new_drop_player_op_c(mut expected: *const TournOp, plyr: PlayerId) {
        let alloc = Box::new(Self::DropPlayer(PlayerIdentifier::Id(plyr)));
        expected = Box::into_raw(alloc);
    }
    
    #[allow(unused_assignments)]
    pub extern "C" fn new_admin_drop_player_op_c(mut expected: *const TournOp, plyr: PlayerId) {
        let alloc = Box::new(Self::AdminDropPlayer(PlayerIdentifier::Id(plyr)));
        expected = Box::into_raw(alloc);
    }
    
    #[allow(unused_assignments)]
    pub extern "C" fn new_add_deck_op_c(
        mut expected: *const TournOp,
        plyr: PlayerId,
        name: *mut c_char,
        // TODO: Make a deck wrapper or make this take a string/url and create a deck
        //deck: Deck,
    ) {
        todo!()
    }
    
    /// Returns 0 if everything is ok.
    /// Returns 1 if there is a name conversion error
    #[allow(unused_assignments)]
    pub extern "C" fn new_remove_deck_op_c(
        mut expected: *const TournOp,
        plyr: PlayerId,
        name_buf: *mut c_char,
    ) -> usize {
        let name_str = unsafe { CStr::from_ptr(name_buf) };
        let name = match name_str.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                return 1;
            }
        };
        let alloc = Box::new(Self::RemoveDeck(PlayerIdentifier::Id(plyr), name));
        expected = Box::into_raw(alloc);
        0
    }
    
    #[allow(unused_assignments)]
    pub extern "C" fn new_set_gamer_tag_op_c(
        mut expected: *const TournOp,
        plyr: PlayerId,
        name_buf: *mut c_char,
    ) -> usize {
        let name_str = unsafe { CStr::from_ptr(name_buf) };
        let name = match name_str.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                return 1;
            }
        };
        let alloc = Box::new(Self::SetGamerTag(PlayerIdentifier::Id(plyr), name));
        expected = Box::into_raw(alloc);
        0
    }
    
    #[allow(unused_assignments)]
    pub extern "C" fn new_ready_player_op_c(mut expected: *const TournOp, plyr: PlayerId) {
        let alloc = Box::new(Self::ReadyPlayer(PlayerIdentifier::Id(plyr)));
        expected = Box::into_raw(alloc);
    }
    
    #[allow(unused_assignments)]
    pub extern "C" fn new_un_ready_player_op_c(mut expected: *const TournOp, plyr: PlayerId) {
        let alloc = Box::new(Self::UnReadyPlayer(PlayerIdentifier::Id(plyr)));
        expected = Box::into_raw(alloc);
    }
    
    // TODO: Make C constructors for tournament settings
    #[allow(improper_ctypes_definitions, unused_assignments)]
    pub extern "C" fn new_update_tourn_setting_op_c(
        mut expected: *const TournOp,
        setting: *const TournamentSetting,
    ) {
        // Safty check: The C side should never construct a settings object and should get it from
        // the Rust side. Thus, the settings are valid. We can *not* ensure that settings isn't a
        // nullptr. We must assume that the C side is diligent.
        // Notable, if the C side gets a pointer to setting from the Rust side and doesn't mess
        // with it, it will always be valid.
        let alloc = Box::new(Self::UpdateTournSetting(unsafe { (*setting).clone() }));
        expected = Box::into_raw(alloc);
    }
    
    #[allow(unused_assignments)]
    pub extern "C" fn new_give_bye_op_c(mut expected: *const TournOp, plyr: PlayerId) {
        let alloc = Box::new(Self::GiveBye(PlayerIdentifier::Id(plyr)));
        expected = Box::into_raw(alloc);
    }
    
    #[allow(unused_assignments)]
    pub extern "C" fn new_create_round_op_c(mut expected: *const TournOp, plyrs: Vec<PlayerId>) {
        todo!()
    }
    
    #[allow(unused_assignments)]
    pub extern "C" fn new_pair_round_op_c(mut expected: *const TournOp) {
        let alloc = Box::new(Self::PairRound());
        expected = Box::into_raw(alloc);
    }
}

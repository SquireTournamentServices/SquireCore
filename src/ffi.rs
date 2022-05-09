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
    //str::Utf8Error,
    time::Duration,
};

#[repr(C)]
pub enum FFIError {
    Utf8Error,
}

#[repr(C)]
pub enum FFIOrTournError {
    FFI(FFIError),
    Tourn(TournamentError),
}

impl Tournament {
    #[no_mangle]
    pub extern "C" fn from_preset_c(
        mut expected: *mut Self,
        name_buf: *mut c_char,
        preset: TournamentPreset,
        format_buf: *mut c_char,
    ) {
        let name_str = unsafe { CStr::from_ptr(name_buf) };
        let name = match name_str.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                return ();
            }
        };
        let format_str = unsafe { CStr::from_ptr(format_buf) };
        let format = match format_str.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                return ();
            }
        };
        expected = &mut Tournament::from_preset(name, preset, format);
    }

    #[no_mangle]
    pub extern "C" fn apply_op_c(&mut self, mut error: *const TournamentError, op: TournOp) {
        match self.apply_op(op) {
            Ok(_) => { return (); },
            Err(e) => { error = &e; return (); },
        }
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

    #[no_mangle]
    pub extern "C" fn get_player_c(&self, mut expected: *const Player, ident: &PlayerIdentifier) {
        match self.get_player(ident) {
            Ok(p) => { expected = &p; },
            Err(_) => { (); },
        }
    }

    #[no_mangle]
    pub extern "C" fn get_round_c(&self, mut expected: *const Round, ident: &RoundIdentifier) {
        match self.get_round(ident) {
            Ok(m) => { expected = &m; },
            Err(_) => { (); },
        }
    }

    #[no_mangle]
    pub fn get_player_deck_c(
        &self,
        expected: *const Deck,
        ident: &PlayerIdentifier,
        name: CString,
    ) {
        let rust_name = match name.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                return ();
            }
        };
        match self.get_player_deck(ident, rust_name) {
            Ok(deck) => { let expected = deck; },
            Err(e) => { return (); },
        }
    }

    #[no_mangle]
    pub extern "C" fn get_player_round_c(&self, mut expected: *const RoundId, ident: &PlayerIdentifier) {
        match self.get_player_round(ident) {
            Ok(r) => { expected = &r; },
            Err(_) => { (); }
        }
    }

    #[no_mangle]
    pub extern "C" fn get_standings_c(&self) -> Standings<StandardScore> {
        self.get_standings()
    }
}

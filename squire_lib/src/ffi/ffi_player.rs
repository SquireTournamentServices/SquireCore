use std::os::raw::c_char;

use crate::{
    ffi::{clone_string_to_c_string, copy_to_system_pointer, print_err, SQUIRE_RUNTIME},
    identifiers::{PlayerId, RoundId, TournamentId},
    players::PlayerStatus,
};

impl PlayerId {
    /// Returns the player name if they can be found
    /// NULL is returned on error or, failure to find
    #[no_mangle]
    pub extern "C" fn pid_name(self, tid: TournamentId) -> *const c_char {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .player_query(tid, self, |p| clone_string_to_c_string(&p.name))
        {
            Ok(name) => name,
            Err(err) => {
                print_err(err, "getting player's name.");
                std::ptr::null()
            }
        }
    }

    /// Returns the player's game name if they can be found
    /// NULL is returned on error or, failure to find
    #[no_mangle]
    pub extern "C" fn pid_game_name(self, tid: TournamentId) -> *const c_char {
        match SQUIRE_RUNTIME.get().unwrap().player_query(tid, self, |p| {
            clone_string_to_c_string(
                p.game_name
                    .as_ref()
                    .map(|n| n.as_str())
                    .unwrap_or_else(|| "No gamer tag"),
            )
        }) {
            Ok(name) => name,
            Err(err) => {
                print_err(err, "getting player's gamer tag.");
                std::ptr::null()
            }
        }
    }

    /// Returns the player's status if they can be found
    /// Dropped on error.
    #[no_mangle]
    pub extern "C" fn pid_status(self, tid: TournamentId) -> PlayerStatus {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .player_query(tid, self, |p| p.status)
        {
            Ok(status) => status,
            Err(err) => {
                print_err(err, "getting player's status.");
                PlayerStatus::Dropped
            }
        }
    }

    /// Returns a raw pointer to rounds that a player is in
    /// This is an array that is terminated by the NULL UUID
    /// This is heap allocted, please free it
    /// Returns NULL on error
    #[no_mangle]
    pub extern "C" fn pid_rounds(self, tid: TournamentId) -> *const RoundId {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .tournament_query(tid, |t| unsafe {
                copy_to_system_pointer(t.round_reg.get_round_ids_for_player(self).into_iter())
            }) {
            Ok(rnds) => rnds,
            Err(err) => {
                print_err(err, "getting player's matches.");
                std::ptr::null()
            }
        }
    }
}

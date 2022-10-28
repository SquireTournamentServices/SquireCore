use std::os::raw::c_char;

use crate::{
    ffi::{clone_string_to_c_string, copy_to_system_pointer, FFI_TOURNAMENT_REGISTRY},
    identifiers::{PlayerId, RoundId, TournamentId},
    player::{Player, PlayerStatus},
};

impl PlayerId {
    /// Returns the player if it can be found in the tournament
    fn get_tourn_player(self, tid: TournamentId) -> Option<Player> {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&tid) {
            Some(t) => t.player_reg.get_player(&self.into()).ok().cloned(),
            None => {
                println!(
                    "[FFI]: Cannot find tournament '{}' during call from PlayerId",
                    *tid
                );
                None
            }
        }
    }

    /// Returns the player name if they can be found
    /// NULL is returned on error or, failure to find
    #[no_mangle]
    pub extern "C" fn pid_name(self, tid: TournamentId) -> *const c_char {
        self.get_tourn_player(tid)
            .map(|p| clone_string_to_c_string(&p.name))
            .unwrap_or_else(|| std::ptr::null::<i8>() as *mut i8)
    }

    /// Returns the player's game name if they can be found
    /// NULL is returned on error or, failure to find
    #[no_mangle]
    pub extern "C" fn pid_game_name(self, tid: TournamentId) -> *const c_char {
        self.get_tourn_player(tid)
            .map(|p| p.game_name)
            .flatten()
            .map(|n| clone_string_to_c_string(&n))
            .unwrap_or_else(|| std::ptr::null::<i8>() as *mut i8)
    }

    /// Returns the player's status if they can be found
    /// Dropped on error.
    #[no_mangle]
    pub extern "C" fn pid_status(self, tid: TournamentId) -> PlayerStatus {
        self.get_tourn_player(tid)
            .map(|p| p.status)
            .unwrap_or(PlayerStatus::Dropped)
    }

    /// Returns a raw pointer to rounds that a player is in
    /// This is an array that is terminated by the NULL UUID
    /// This is heap allocted, please free it
    /// Returns NULL on error
    #[no_mangle]
    pub extern "C" fn pid_rounds(self: Self, tid: TournamentId) -> *const RoundId {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&tid) {
            Some(t) => unsafe {
                copy_to_system_pointer(t.round_reg.get_round_ids_for_player(self).into_iter())
            },
            None => std::ptr::null(),
        }
    }
}

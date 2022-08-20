use std::os::raw::c_char;

use crate::{
    ffi::{clone_string_to_c_string, FFI_TOURNAMENT_REGISTRY},
    identifiers::{PlayerId, TournamentId},
    player::{Player, PlayerStatus},
};

impl PlayerId {
    /// Returns the player if it can be found in the tournament
    fn get_tourn_player(self, tid: TournamentId) -> Option<Player> {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&tid) {
            // TODO: Get rid of this extra clone
            Some(t) => t.player_reg.get_player(&self.into()).cloned(),
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
            .map(|p| clone_string_to_c_string(p.name.clone()))
            .unwrap_or_else(|| std::ptr::null::<i8>() as *mut i8)
    }

    /// Returns the player's game name if they can be found
    /// NULL is returned on error or, failure to find
    #[no_mangle]
    pub extern "C" fn pid_game_name(self, tid: TournamentId) -> *const c_char {
        if let Some(Some(name)) = self.get_tourn_player(tid).map(|p| p.game_name) {
            clone_string_to_c_string(name)
        } else {
            std::ptr::null()
        }
    }

    /// Returns the player's status if they can be found
    /// Dropped on error.
    #[no_mangle]
    pub extern "C" fn pid_status(self, tid: TournamentId) -> PlayerStatus {
        self.get_tourn_player(tid)
            .map(|p| p.status)
            .unwrap_or(PlayerStatus::Dropped)
    }
}

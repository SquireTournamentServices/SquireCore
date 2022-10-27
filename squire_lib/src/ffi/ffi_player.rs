use crate::{
    ffi::{clone_string_to_c_string, FFI_TOURNAMENT_REGISTRY},
    identifiers::{PlayerId, RoundId, TournamentId},
    player::{Player, PlayerStatus},
};
use std::{
    alloc::{Allocator, Layout, System},
    os::raw::c_char,
    ptr,
};
use uuid::Uuid;

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
        let tourn = match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&tid) {
            Some(t) => t,
            None => {
                return std::ptr::null();
            }
        };

        let rounds = tourn.round_reg.get_round_ids_for_player(self);
        let length = rounds.len();

        let len = (length + 1) * std::mem::size_of::<RoundId>();
        let ptr = System
            .allocate(Layout::from_size_align(len, 1).unwrap())
            .unwrap()
            .as_mut_ptr() as *mut RoundId;
        let slice = unsafe { &mut *(ptr::slice_from_raw_parts(ptr, len) as *mut [RoundId]) };
        slice.iter_mut().zip(rounds).for_each(|(dst, c)| {
            *dst = c;
        });
        slice[length] = Uuid::default().into();
        return ptr;
    }
}

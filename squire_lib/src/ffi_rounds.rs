use crate::ffi::FFI_TOURNAMENT_REGISTRY;
use crate::{
    identifiers::{PlayerId, RoundId, TournamentId},
    round::{Round, RoundStatus},
};
use std::alloc::{Allocator, Layout, System};
use std::collections::HashSet;
use std::option::Option;
use std::ptr;
use uuid::Uuid;

impl RoundId {
    /// Returns the round if it can be found in the tournament
    fn get_tourn_round(self, tid: TournamentId) -> Option<Round> {
        unsafe {
            match FFI_TOURNAMENT_REGISTRY.get_mut().unwrap().get(&tid) {
                Some(t) => {
                    return t.round_reg.get_round(&self.into()).cloned();
                }
                None => {
                    println!("[FFI]: Cannot find tournament in get_tourn_round();");
                    return None;
                }
            }
        }
    }

    /// Gets the round number
    /// -1 on error
    #[no_mangle]
    pub extern "C" fn rid_match_number(self, tid: TournamentId) -> i64 {
        match self.get_tourn_round(tid) {
            Some(r) => {
                return r.match_number as i64;
            }
            None => {
                return -1;
            }
        }
    }

    /// Gets the table number
    /// Warning: Currently cannot detect if the round has not been allocated a table number
    /// -1 on error
    #[no_mangle]
    pub extern "C" fn rid_table_number(self, tid: TournamentId) -> i64 {
        match self.get_tourn_round(tid) {
            Some(r) => {
                return r.table_number as i64;
            }
            None => {
                return -1;
            }
        }
    }

    /// Gets the status for a round
    /// Dead on error
    #[no_mangle]
    pub extern "C" fn rid_status(self, tid: TournamentId) -> RoundStatus {
        match self.get_tourn_round(tid) {
            Some(r) => {
                return r.status;
            }
            None => {
                return RoundStatus::Dead;
            }
        }
    }

    /// Returns the amount of time left in a round
    /// Retrusn -1 on error
    #[no_mangle]
    pub extern "C" fn rid_time_left(self, tid: TournamentId) -> i64 {
        match self.get_tourn_round(tid) {
            Some(r) => {
                return r.time_left().as_secs() as i64;
            }
            None => {
                return -1;
            }
        }
    }

    /// Returns the total duration in a round
    /// Retrusn -1 on error
    #[no_mangle]
    pub extern "C" fn rid_duration(self, tid: TournamentId) -> i64 {
        match self.get_tourn_round(tid) {
            Some(r) => {
                return r.length.as_secs() as i64;
            }
            None => {
                return -1;
            }
        }
    }

    /// Gets the players that are in a round
    /// Returns NULL on error
    #[no_mangle]
    pub extern "C" fn rid_players(self, tid: TournamentId) -> *const PlayerId {
        match self.get_tourn_round(tid) {
            Some(r) => {
                let players: HashSet<PlayerId> = r.players;
                let len: usize = (players.len() + 1) * std::mem::size_of::<PlayerId>();

                unsafe {
                    let ptr = System
                        .allocate(Layout::from_size_align(len, 1).unwrap())
                        .unwrap()
                        .as_mut_ptr() as *mut PlayerId;
                    let slice = &mut *(ptr::slice_from_raw_parts(ptr, len) as *mut [PlayerId]);

                    let mut i: usize = 0;
                    for p in players {
                        slice[i] = p;
                        i += 1;
                    }
                    slice[i] = Uuid::default().into();
                    return ptr;
                }
            }
            None => {
                return std::ptr::null();
            }
        }
    }

    /// Gets the players that are in a round who have confirmed
    /// Returns NULL on error
    #[no_mangle]
    pub extern "C" fn rid_confirmed_players(self, tid: TournamentId) -> *const PlayerId {
        match self.get_tourn_round(tid) {
            Some(r) => {
                let players: HashSet<PlayerId> = r.confirmations;
                let len: usize = (players.len() + 1) * std::mem::size_of::<PlayerId>();

                unsafe {
                    let ptr = System
                        .allocate(Layout::from_size_align(len, 1).unwrap())
                        .unwrap()
                        .as_mut_ptr() as *mut PlayerId;
                    let slice = &mut *(ptr::slice_from_raw_parts(ptr, len) as *mut [PlayerId]);

                    let mut i: usize = 0;
                    for p in players {
                        slice[i] = p;
                        i += 1;
                    }
                    slice[i] = Uuid::default().into();
                    return ptr;
                }
            }
            None => {
                return std::ptr::null();
            }
        }
    }
}

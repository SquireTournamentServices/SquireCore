use crate::ffi::FFI_TOURNAMENT_REGISTRY;
use crate::{
    identifiers::{AdminId, PlayerId, RoundId, RoundIdentifier, TournamentId},
    operations::TournOp,
    round::{Round, RoundResult, RoundStatus},
};
use std::alloc::{Allocator, Layout, System};
use std::collections::HashSet;
use std::mem;
use std::option::Option;
use std::ptr;
use uuid::Uuid;

impl RoundId {
    /// Returns the round if it can be found in the tournament
    fn get_tourn_round(self, tid: TournamentId) -> Option<Round> {
        unsafe {
            match FFI_TOURNAMENT_REGISTRY.get_mut().unwrap().get(&tid) {
                Some(t) => {
                    return match t.round_reg.get_round(&self.into()).cloned() {
                        Ok(r) => Some(r),
                        Err(e) => {
                            println!("[FFI]: Cannot find tournament in get_tourn_round() {};", e);
                            None
                        }
                    }
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
        self.get_tourn_round(tid)
            .map(|r| r.match_number as i64)
            .unwrap_or(-1)
    }

    /// Gets the table number
    /// Warning: Currently cannot detect if the round has not been allocated a table number
    /// -1 on error
    #[no_mangle]
    pub extern "C" fn rid_table_number(self, tid: TournamentId) -> i64 {
        self.get_tourn_round(tid)
            .map(|r| r.table_number as i64)
            .unwrap_or(-1)
    }

    /// Gets the status for a round
    /// Dead on error
    #[no_mangle]
    pub extern "C" fn rid_status(self, tid: TournamentId) -> RoundStatus {
        self.get_tourn_round(tid)
            .map(|r| r.status)
            .unwrap_or(RoundStatus::Dead)
    }

    /// Returns the amount of time left in a round
    /// Retrusn -1 on error
    #[no_mangle]
    pub extern "C" fn rid_time_left(self, tid: TournamentId) -> i64 {
        self.get_tourn_round(tid)
            .map(|r| r.time_left().as_secs() as i64)
            .unwrap_or(-1)
    }

    /// Returns the total duration in a round
    /// Retrusn -1 on error
    #[no_mangle]
    pub extern "C" fn rid_duration(self, tid: TournamentId) -> i64 {
        self.get_tourn_round(tid)
            .map(|r| r.length.as_secs() as i64)
            .unwrap_or(-1)
    }

    /// Gets the players that are in a round
    /// Returns NULL on error
    #[no_mangle]
    pub extern "C" fn rid_players(self, tid: TournamentId) -> *const PlayerId {
        match self.get_tourn_round(tid) {
            Some(r) => {
                let players: HashSet<PlayerId> = r.players;
                let len: usize = (players.len() + 1) * std::mem::size_of::<PlayerId>();

                let ptr = System
                    .allocate(Layout::from_size_align(len, 1).unwrap())
                    .unwrap()
                    .as_mut_ptr() as *mut PlayerId;
                let slice =
                    unsafe { &mut *(ptr::slice_from_raw_parts(ptr, len) as *mut [PlayerId]) };

                let mut i: usize = 0;
                for p in players {
                    slice[i] = p;
                    i += 1;
                }
                slice[i] = Uuid::default().into();
                return ptr;
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

                let ptr = System
                    .allocate(Layout::from_size_align(len, 1).unwrap())
                    .unwrap()
                    .as_mut_ptr() as *mut PlayerId;
                let slice =
                    unsafe { &mut *(ptr::slice_from_raw_parts(ptr, len) as *mut [PlayerId]) };

                let mut i: usize = 0;
                for p in players {
                    slice[i] = p;
                    i += 1;
                }
                slice[i] = Uuid::default().into();
                return ptr;
            }
            None => {
                return std::ptr::null();
            }
        }
    }

    /// Confirms a result
    #[no_mangle]
    pub unsafe extern "C" fn rid_record_result(
        self,
        tid: TournamentId,
        aid: AdminId,
        result: RoundResult,
    ) -> bool {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get_mut(&tid) {
            Some(mut tournament) => {
                match tournament.apply_op(TournOp::AdminRecordResult(
                    aid.into(),
                    RoundIdentifier::Id(self),
                    result,
                )) {
                    Err(err) => {
                        println!("[FFI]: ffi_record_result error {}", err);
                        false
                    }
                    Ok(_) => true,
                }
            }
            None => false,
        }
    }

    #[no_mangle]
    pub extern "C" fn rid_results(self, tid: TournamentId) -> *const RoundResult {
        let mut r: Round = match self.get_tourn_round(tid) {
            Some(round) => round,
            None => {
                return std::ptr::null();
            }
        };

        let mut results: Vec<RoundResult> = Vec::<RoundResult>::new();
        for pid in r.results {
            results.push(RoundResult::Wins(pid.0, pid.1));
        }

        if r.draws > 0 {
            results.push(RoundResult::Draw(r.draws));
        }

        let len: usize = (results.len() + 1) * std::mem::size_of::<RoundResult>();

        let ptr = System
            .allocate(Layout::from_size_align(len, 1).unwrap())
            .unwrap()
            .as_mut_ptr() as *mut RoundResult;
        let slice = unsafe { &mut *(ptr::slice_from_raw_parts(ptr, len) as *mut [RoundResult]) };

        let mut i: usize = 0;
        for r in results {
            slice[i] = r;
            i += 1;
        }
        slice[i] = unsafe { mem::zeroed() };
        return ptr;
    }
}

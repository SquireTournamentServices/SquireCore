use crate::ffi::FFI_TOURNAMENT_REGISTRY;
use crate::{
    identifiers::{AdminId, PlayerId, RoundId, RoundIdentifier, TournamentId},
    operations::{OpData, TournOp},
    round::{Round, RoundResult, RoundStatus},
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

    /// Confirms a player for the match result
    /// false on error
    #[no_mangle]
    pub unsafe extern "C" fn rid_confirm_player(
        self,
        tid: TournamentId,
        aid: AdminId,
        pid: PlayerId,
    ) -> bool {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get_mut(&tid) {
            Some(mut tournament) => {
                match tournament.apply_op(TournOp::AdminConfirmResult(
                    aid.into(),
                    RoundIdentifier::Id(self),
                    pid.into(),
                )) {
                    Err(err) => {
                        println!("[FFI]: rid_confirm_player error {}", err);
                        false
                    }
                    Ok(OpData::ConfirmResult(_, _)) => true,
                    Ok(_) => {
                        println!("[FFI]: rid_confirm_player unexpected result");
                        false
                    }
                }
            }
            None => false,
        }
    }

    /// Records results for a round; DO NOT RECORD DRAWS HERE (it breaks :( )
    /// false on error
    #[no_mangle]
    pub unsafe extern "C" fn rid_record_result(
        self,
        tid: TournamentId,
        aid: AdminId,
        pid: PlayerId,
        wins: u32,
    ) -> bool {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get_mut(&tid) {
            Some(mut tournament) => {
                match tournament.apply_op(TournOp::AdminRecordResult(
                    aid.into(),
                    RoundIdentifier::Id(self),
                    RoundResult::Wins(pid, wins),
                )) {
                    Err(err) => {
                        println!("[FFI]: rid_record_result error {}", err);
                        false
                    }
                    Ok(_) => true,
                }
            }
            None => false,
        }
    }

    /// Records draws for a round
    #[no_mangle]
    pub unsafe extern "C" fn rid_record_draws(
        self,
        tid: TournamentId,
        aid: AdminId,
        draws: u32,
    ) -> bool {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get_mut(&tid) {
            Some(mut tournament) => {
                match tournament.apply_op(TournOp::AdminRecordResult(
                    aid.into(),
                    RoundIdentifier::Id(self),
                    RoundResult::Draw(draws),
                )) {
                    Err(err) => {
                        println!("[FFI]: ffi_record_draw error {}", err);
                        false
                    }
                    Ok(_) => true,
                }
            }
            None => false,
        }
    }

    /// Returns the draw count for a game
    /// Returns -1 on error
    #[no_mangle]
    pub extern "C" fn rid_draws(self, tid: TournamentId) -> i32 {
        match self.get_tourn_round(tid) {
            Some(round) => round.draws as i32,
            None => -1,
        }
    }

    /// Returns the result for a player in a round
    /// Returns -1 on error
    #[no_mangle]
    pub extern "C" fn rid_result_for(self, tid: TournamentId, pid: PlayerId) -> i32 {
        match self.get_tourn_round(tid) {
            Some(round) => match round.results.get(&pid) {
                Some(wins) => *wins as i32,
                None => {
                    println!("[FFI]: Cannot find player in rid_result_for");
                    -1
                }
            },
            None => -1,
        }
    }

    /// Kills a match
    /// false on error
    #[no_mangle]
    pub unsafe extern "C" fn rid_kill(self, tid: TournamentId, aid: AdminId) -> bool {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get_mut(&tid) {
            Some(mut tournament) => {
                match tournament
                    .apply_op(TournOp::RemoveRound(aid.into(), RoundIdentifier::Id(self)))
                {
                    Err(err) => {
                        println!("[FFI]: rid_kill error {}", err);
                        false
                    }
                    Ok(_) => true,
                }
            }
            None => false,
        }
    }
}

use crate::{
    ffi::{copy_to_system_pointer, FFI_TOURNAMENT_REGISTRY},
    identifiers::{AdminId, PlayerId, RoundId, TournamentId},
    operations::{AdminOp, JudgeOp, OpData, TournOp},
    rounds::{Round, RoundResult, RoundStatus},
};

impl RoundId {
    /// Returns the round if it can be found in the tournament
    fn get_tourn_round(self, tid: TournamentId) -> Option<Round> {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&tid) {
            Some(t) => match t.round_reg.get_round(&self.into()).cloned() {
                Ok(r) => Some(r),
                Err(e) => {
                    println!("[FFI]: Cannot find round in get_tourn_round(): Error {e};");
                    None
                }
            },
            None => {
                println!("[FFI]: Cannot find tournament in get_tourn_round();");
                None
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
            Some(r) => unsafe { copy_to_system_pointer(r.players.into_iter()) },
            None => std::ptr::null(),
        }
    }

    /// Gets the players that are in a round who have confirmed
    /// Returns NULL on error
    #[no_mangle]
    pub extern "C" fn rid_confirmed_players(self, tid: TournamentId) -> *const PlayerId {
        match self.get_tourn_round(tid) {
            Some(r) => unsafe { copy_to_system_pointer(r.confirmations.into_iter()) },
            None => std::ptr::null(),
        }
    }

    /// Confirms a player for the match result
    /// false on error
    #[no_mangle]
    pub extern "C" fn rid_confirm_player(
        self,
        tid: TournamentId,
        aid: AdminId,
        pid: PlayerId,
    ) -> bool {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get_mut(&tid) {
            Some(mut tournament) => {
                match tournament.apply_op(TournOp::JudgeOp(
                    aid.into(),
                    JudgeOp::AdminConfirmResult(self, pid.into()),
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
    pub extern "C" fn rid_record_result(
        self,
        tid: TournamentId,
        aid: AdminId,
        pid: PlayerId,
        wins: u32,
    ) -> bool {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get_mut(&tid) {
            Some(mut tournament) => {
                match tournament.apply_op(TournOp::JudgeOp(
                    aid.into(),
                    JudgeOp::AdminRecordResult(self, RoundResult::Wins(pid, wins)),
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
    pub extern "C" fn rid_record_draws(self, tid: TournamentId, aid: AdminId, draws: u32) -> bool {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get_mut(&tid) {
            Some(mut tournament) => {
                match tournament.apply_op(TournOp::JudgeOp(
                    aid.into(),
                    JudgeOp::AdminRecordResult(self, RoundResult::Draw(draws)),
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
    pub extern "C" fn rid_kill(self, tid: TournamentId, aid: AdminId) -> bool {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get_mut(&tid) {
            Some(mut tournament) => {
                match tournament.apply_op(TournOp::AdminOp(aid, AdminOp::RemoveRound(self))) {
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

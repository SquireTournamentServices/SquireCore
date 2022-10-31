use crate::{
    ffi::{copy_to_system_pointer, print_err, SQUIRE_RUNTIME},
    identifiers::{AdminId, PlayerId, RoundId, TournamentId},
    operations::{AdminOp, JudgeOp, TournOp},
    rounds::{RoundResult, RoundStatus},
};


impl RoundId {
    /// Gets the round number
    /// -1 on error
    #[no_mangle]
    pub extern "C" fn rid_match_number(self, tid: TournamentId) -> i64 {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .round_query(tid, self, |r| r.match_number as i64)
        {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting round's match number.");
                -1
            }
        }
    }

    /// Gets the table number
    /// Warning: Currently cannot detect if the round has not been allocated a table number
    /// -1 on error
    #[no_mangle]
    pub extern "C" fn rid_table_number(self, tid: TournamentId) -> i64 {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .round_query(tid, self, |r| r.table_number as i64)
        {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting round's table number.");
                -1
            }
        }
    }

    /// Gets the status for a round
    /// Dead on error
    #[no_mangle]
    pub extern "C" fn rid_status(self, tid: TournamentId) -> RoundStatus {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .round_query(tid, self, |r| r.status)
        {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting round's status.");
                RoundStatus::Dead
            }
        }
    }

    /// Returns the amount of time left in a round
    /// Retrusn -1 on error
    #[no_mangle]
    pub extern "C" fn rid_time_left(self, tid: TournamentId) -> i64 {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .round_query(tid, self, |r| r.time_left().as_secs() as i64)
        {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting round's remaining time.");
                -1
            }
        }
    }

    /// Returns the total duration in a round
    /// Retrusn -1 on error
    #[no_mangle]
    pub extern "C" fn rid_duration(self, tid: TournamentId) -> i64 {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .round_query(tid, self, |r| r.length.as_secs() as i64)
        {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting round's length.");
                -1
            }
        }
    }

    /// Gets the players that are in a round
    /// Returns NULL on error
    #[no_mangle]
    pub extern "C" fn rid_players(self, tid: TournamentId) -> *const PlayerId {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .round_query(tid, self, |r| unsafe {
                copy_to_system_pointer(r.players.iter().cloned())
            }) {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting round's players.");
                std::ptr::null()
            }
        }
    }

    /// Gets the players that are in a round who have confirmed
    /// Returns NULL on error
    #[no_mangle]
    pub extern "C" fn rid_confirmed_players(self, tid: TournamentId) -> *const PlayerId {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .round_query(tid, self, |r| unsafe {
                copy_to_system_pointer(r.confirmations.iter().cloned())
            }) {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting round's confirmed players.");
                std::ptr::null()
            }
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
        match SQUIRE_RUNTIME.get().unwrap().apply_operation(
            tid,
            TournOp::JudgeOp(aid.into(), JudgeOp::AdminConfirmResult(self, pid.into())),
        ) {
            Ok(_) => true,
            Err(err) => {
                print_err(err, "getting confirming player.");
                false
            }
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
        match SQUIRE_RUNTIME.get().unwrap().apply_operation(
            tid,
            TournOp::JudgeOp(
                aid.into(),
                JudgeOp::AdminRecordResult(self, RoundResult::Wins(pid, wins)),
            ),
        ) {
            Ok(_) => true,
            Err(err) => {
                print_err(err, "recording a match result.");
                false
            }
        }
    }

    /// Records draws for a round
    #[no_mangle]
    pub extern "C" fn rid_record_draws(self, tid: TournamentId, aid: AdminId, draws: u32) -> bool {
        match SQUIRE_RUNTIME.get().unwrap().apply_operation(
            tid,
            TournOp::JudgeOp(
                aid.into(),
                JudgeOp::AdminRecordResult(self, RoundResult::Draw(draws)),
            ),
        ) {
            Ok(_) => true,
            Err(err) => {
                print_err(err, "getting a draws.");
                false
            }
        }
    }

    /// Returns the draw count for a game
    /// Returns -1 on error
    #[no_mangle]
    pub extern "C" fn rid_draws(self, tid: TournamentId) -> i32 {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .round_query(tid, self, |r| r.draws as i32)
        {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting round's number of draws.");
                -1
            }
        }
    }

    /// Returns the result for a player in a round
    /// Returns -1 on error
    #[no_mangle]
    pub extern "C" fn rid_result_for(self, tid: TournamentId, pid: PlayerId) -> i32 {
        match SQUIRE_RUNTIME.get().unwrap().round_query(tid, self, |r| {
            r.results.get(&pid).map(|r| *r as i32).unwrap_or(-1)
        }) {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting round's number of result.");
                -1
            }
        }
    }

    /// Kills a match
    /// false on error
    #[no_mangle]
    pub extern "C" fn rid_kill(self, tid: TournamentId, aid: AdminId) -> bool {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .apply_operation(tid, TournOp::AdminOp(aid, AdminOp::RemoveRound(self)))
        {
            Ok(_) => true,
            Err(err) => {
                print_err(err, "killing the round.");
                false
            }
        }
    }
}

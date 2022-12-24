use std::{ffi::CStr, os::raw::c_char, time::Duration};

use serde_json;
use uuid::Uuid;

use crate::{
    accounts::SquireAccount,
    admin::Admin,
    ffi::{clone_string_to_c_string, copy_to_system_pointer, print_err, SQUIRE_RUNTIME},
    identifiers::{AdminId, PlayerId, RoundId, SquireAccountId, TournamentId},
    operations::{AdminOp, OpData, TournOp},
    pairings::PairingStyle,
    scoring::StandardScore,
    settings::{PairingSetting, TournamentSetting},
    tournament::{Tournament, TournamentPreset, TournamentStatus},
};

const BACKUP_EXT: &str = ".bak";

#[repr(C)]
#[derive(Debug, Default, Clone)]
/// A struct used to pass scores to scores across the language boundary
pub struct PlayerScore<S> {
    pid: PlayerId,
    score: S,
}

/// TournamentIds can be used to get data safely from
/// the Rust lib with these methods
impl TournamentId {
    /// Returns a raw pointer to a list of standings
    /// This is an array, the last element has a NULL player id
    /// This is heap allocated, please free it
    /// Returns NULL on error
    #[no_mangle]
    pub extern "C" fn tid_standings(self) -> *const PlayerScore<StandardScore> {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .tournament_query(self, |t| unsafe {
                copy_to_system_pointer(
                    t.get_standings()
                        .scores
                        .into_iter()
                        .map(|(pid, score)| PlayerScore { pid, score }),
                )
            }) {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting standings.");
                std::ptr::null()
            }
        }
    }

    /// Returns a raw pointer to players
    /// This is an array that is terminated by the NULL UUID
    /// This is heap allocted, please free it
    /// Returns NULL on error
    #[no_mangle]
    pub extern "C" fn tid_players(self) -> *const PlayerId {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .tournament_query(self, |t| unsafe {
                copy_to_system_pointer(t.player_reg.players.keys().cloned())
            }) {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "players.");
                std::ptr::null()
            }
        }
    }

    /// Returns a raw pointer to rounds
    /// This is an array that is terminated by the NULL UUID
    /// This is heap allocted, please free it
    /// Returns NULL on error
    #[no_mangle]
    pub extern "C" fn tid_rounds(self) -> *const RoundId {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .tournament_query(self, |t| unsafe {
                copy_to_system_pointer(t.round_reg.num_and_id.iter_right().cloned())
            }) {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "rounds.");
                std::ptr::null()
            }
        }
    }

    /// Adds a player to a tournament
    /// On error a NULL UUID is returned
    #[no_mangle]
    pub extern "C" fn tid_add_player(self, __name: *const c_char) -> PlayerId {
        let name = unsafe { CStr::from_ptr(__name).to_str().unwrap() };
        let op = TournOp::RegisterPlayer(SquireAccount::new(name.to_string(), name.to_string()));

        match SQUIRE_RUNTIME.get().unwrap().apply_operation(self, op) {
            Ok(data) => data.assume_register_player(),
            Err(err) => {
                print_err(err, "adding player.");
                PlayerId::default()
            }
        }
    }

    #[no_mangle]
    /// Drops a player for the tournament
    /// On error false is returned
    pub extern "C" fn tid_drop_player(self, pid: PlayerId, aid: AdminId) -> bool {
        match SQUIRE_RUNTIME.get().unwrap().apply_operation(
            self,
            TournOp::AdminOp(aid, AdminOp::AdminDropPlayer(pid.into())),
        ) {
            Ok(_) => true,
            Err(err) => {
                print_err(err, "dropping player.");
                false
            }
        }
    }

    /// Adds an admin to a local tournament in a way that is perfect for
    /// adding the system user.
    #[no_mangle]
    pub extern "C" fn tid_add_admin_local(
        self,
        __name: *const c_char,
        aid: AdminId,
        uid: SquireAccountId,
    ) -> bool {
        let name = unsafe { CStr::from_ptr(__name).to_str().unwrap() };
        let mut account = SquireAccount::new(name.to_string(), name.to_string());
        account.id = uid;
        let admin = Admin::new(account);

        match SQUIRE_RUNTIME.get().unwrap().mutate_tournament(self, |t| {
            t.admins.insert(aid, admin.clone());
        }) {
            Ok(_) => true,
            Err(err) => {
                print_err(err, "adding admin.");
                false
            }
        }
    }

    /// Defrosts a tournament
    /// false on error, true on success.
    #[no_mangle]
    pub extern "C" fn tid_thaw(self, aid: AdminId) -> bool {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .apply_operation(self, TournOp::AdminOp(aid, AdminOp::Thaw))
        {
            Ok(_) => true,
            Err(err) => {
                print_err(err, "thawing.");
                false
            }
        }
    }

    /// Freezes a tournament
    /// false on error, true on success.
    #[no_mangle]
    pub extern "C" fn tid_freeze(self, aid: AdminId) -> bool {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .apply_operation(self, TournOp::AdminOp(aid, AdminOp::Freeze))
        {
            Ok(_) => true,
            Err(err) => {
                print_err(err, "freezing.");
                false
            }
        }
    }

    /// End a tournament
    /// false on error, true on success.
    #[no_mangle]
    pub extern "C" fn tid_end(self, aid: AdminId) -> bool {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .apply_operation(self, TournOp::AdminOp(aid, AdminOp::End))
        {
            Ok(_) => true,
            Err(err) => {
                print_err(err, "ending.");
                false
            }
        }
    }

    /// Cancels a tournament
    /// false on error, true on success.
    #[no_mangle]
    pub extern "C" fn tid_cancel(self, aid: AdminId) -> bool {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .apply_operation(self, TournOp::AdminOp(aid, AdminOp::Cancel))
        {
            Ok(_) => true,
            Err(err) => {
                print_err(err, "cancelling.");
                false
            }
        }
    }

    /// Starts a tournament
    /// false on error, true on success.
    #[no_mangle]
    pub extern "C" fn tid_start(self, aid: AdminId) -> bool {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .apply_operation(self, TournOp::AdminOp(aid, AdminOp::Start))
        {
            Ok(_) => true,
            Err(err) => {
                print_err(err, "starting.");
                false
            }
        }
    }

    /// Updates the settings
    /// Values that are not to be changed should remain the
    /// current setting, that would be the value the user
    /// selected in the GUI so that is fine.
    /// All input must be non-null.
    ///
    /// If any errors occur then all actions are rolled back
    /// and, false returned.
    ///
    /// Otherwise true is returned and the operations are all
    /// applied to the tournament.
    #[no_mangle]
    pub extern "C" fn tid_update_settings(
        self,
        __format: *const c_char,
        starting_table_number: u64,
        use_table_number: bool,
        game_size: u8,
        min_deck_count: u8,
        max_deck_count: u8,
        match_length: u64,
        reg_open: bool,
        require_check_in: bool,
        require_deck_reg: bool,
        aid: AdminId,
    ) -> bool {
        // Sort input strings out
        let format =
            String::from(unsafe { CStr::from_ptr(__format).to_str().unwrap().to_string() });

        let rt = SQUIRE_RUNTIME.get().unwrap();

        if let Err(err) = rt.apply_operation(
            self,
            TournOp::AdminOp(
                aid,
                AdminOp::UpdateTournSetting(TournamentSetting::Format(format)),
            ),
        ) {
            print_err(err, "updating format.");
            return false;
        }

        if let Err(err) = rt.apply_operation(
            self,
            TournOp::AdminOp(
                aid,
                AdminOp::UpdateTournSetting(TournamentSetting::StartingTableNumber(
                    starting_table_number,
                )),
            ),
        ) {
            print_err(err, "updating starting table number.");
            return false;
        }

        if let Err(err) = rt.apply_operation(
            self,
            TournOp::AdminOp(
                aid,
                AdminOp::UpdateTournSetting(TournamentSetting::UseTableNumbers(use_table_number)),
            ),
        ) {
            print_err(err, "updating use table number.");
            return false;
        }

        if let Err(err) = rt.apply_operation(
            self,
            TournOp::AdminOp(
                aid,
                AdminOp::UpdateTournSetting(TournamentSetting::PairingSetting(
                    PairingSetting::MatchSize(game_size),
                )),
            ),
        ) {
            print_err(err, "updating match size.");
            return false;
        }

        let curr_max = rt.tournament_query(self, |t| t.max_deck_count).unwrap();

        let (deck_op_one, deck_op_two) = if min_deck_count > curr_max {
            (
                AdminOp::UpdateTournSetting(TournamentSetting::MaxDeckCount(max_deck_count)),
                AdminOp::UpdateTournSetting(TournamentSetting::MinDeckCount(min_deck_count)),
            )
        } else {
            (
                AdminOp::UpdateTournSetting(TournamentSetting::MinDeckCount(min_deck_count)),
                AdminOp::UpdateTournSetting(TournamentSetting::MaxDeckCount(max_deck_count)),
            )
        };

        if let Err(err) = rt.apply_operation(self, TournOp::AdminOp(aid, deck_op_one)) {
            print_err(err, "updating deck count.");
            return false;
        }

        if let Err(err) = rt.apply_operation(self, TournOp::AdminOp(aid, deck_op_two)) {
            print_err(err, "updating deck count.");
            return false;
        }

        if let Err(err) = rt.apply_operation(
            self,
            TournOp::AdminOp(
                aid,
                AdminOp::UpdateTournSetting(TournamentSetting::RoundLength(Duration::new(
                    match_length,
                    0,
                ))),
            ),
        ) {
            print_err(err, "updating round length.");
            return false;
        }

        if let Err(err) =
            rt.apply_operation(self, TournOp::AdminOp(aid, AdminOp::UpdateReg(reg_open)))
        {
            print_err(err, "updating regsitration status.");
            return false;
        }

        if let Err(err) = rt.apply_operation(
            self,
            TournOp::AdminOp(
                aid,
                AdminOp::UpdateTournSetting(TournamentSetting::RequireCheckIn(require_check_in)),
            ),
        ) {
            print_err(err, "updating require check in.");
            return false;
        }

        if let Err(err) = rt.apply_operation(
            self,
            TournOp::AdminOp(
                aid,
                AdminOp::UpdateTournSetting(TournamentSetting::RequireDeckReg(require_deck_reg)),
            ),
        ) {
            print_err(err, "updating require deck reg.");
            return false;
        }

        true
    }

    /// Pairs a set of rounds
    /// Returns a null terminated list of the round ids
    /// Returns NULL on error
    #[no_mangle]
    pub extern "C" fn tid_pair_round(self, aid: AdminId) -> *const RoundId {
        let rt = SQUIRE_RUNTIME.get().unwrap();
        match rt.tournament_query(self, |t| t.create_pairings()) {
            Ok(Some(pairings)) => {
                match rt.apply_operation(self, TournOp::AdminOp(aid, AdminOp::PairRound(pairings)))
                {
                    Ok(OpData::Pair(rnds)) => unsafe { copy_to_system_pointer(rnds.into_iter()) },
                    Err(err) => {
                        print_err(err, "pairing round.");
                        std::ptr::null()
                    }
                    _ => std::ptr::null(),
                }
            }
            Err(err) => {
                print_err(err, "pairing round.");
                std::ptr::null()
            }
            Ok(None) => std::ptr::null(),
        }
    }

    /// Returns the name of a tournament
    /// Returns NULL if an error happens
    /// This is heap allocated, please free it
    #[no_mangle]
    pub extern "C" fn tid_name(self) -> *const c_char {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .tournament_query(self, |t| clone_string_to_c_string(&t.name))
        {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting tournament name.");
                std::ptr::null()
            }
        }
    }

    /// Returns the format of a tournament
    /// Returns NULL if an error happens
    /// This is heap allocated, please free it
    #[no_mangle]
    pub extern "C" fn tid_format(self) -> *const c_char {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .tournament_query(self, |t| clone_string_to_c_string(&t.format))
        {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting tournament format.");
                std::ptr::null()
            }
        }
    }

    /// Returns the starting table number
    /// Retruns -1 on error
    #[no_mangle]
    pub extern "C" fn tid_starting_table_number(self) -> i32 {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .tournament_query(self, |t| t.round_reg.starting_table as i32)
        {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting starting table number.");
                -1
            }
        }
    }

    /// Returns whether table numbers are being used for this tournament
    /// false, is the error value (kinda sketchy)
    #[no_mangle]
    pub extern "C" fn tid_use_table_number(self) -> bool {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .tournament_query(self, |t| t.use_table_number)
        {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting use table number.");
                false
            }
        }
    }

    /// Returns the game size
    /// -1 is the error value
    #[no_mangle]
    pub extern "C" fn tid_game_size(self) -> i32 {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .tournament_query(self, |t| t.pairing_sys.match_size as i32)
        {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting match size.");
                -1
            }
        }
    }

    /// Returns the min deck count
    /// -1 is the error value
    #[no_mangle]
    pub extern "C" fn tid_min_deck_count(self) -> i32 {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .tournament_query(self, |t| t.min_deck_count as i32)
        {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting min deck count.");
                -1
            }
        }
    }

    /// Returns the max deck count
    /// -1 is the error value
    #[no_mangle]
    pub extern "C" fn tid_max_deck_count(self) -> i32 {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .tournament_query(self, |t| t.max_deck_count as i32)
        {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting max deck count.");
                -1
            }
        }
    }

    /// Returns the pairing type
    /// This is of type TournamentPreset, but an int to let me return error values
    /// -1 is error value
    #[no_mangle]
    pub extern "C" fn tid_pairing_type(self) -> i32 {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .tournament_query(self, |t| match t.pairing_sys.style {
                PairingStyle::Swiss(_) => TournamentPreset::Swiss as i32,
                PairingStyle::Fluid(_) => TournamentPreset::Fluid as i32,
            }) {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting pairing type.");
                -1
            }
        }
    }

    /// Returns the round length
    /// -1 on error
    #[no_mangle]
    pub extern "C" fn tid_round_length(self) -> i64 {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .tournament_query(self, |t| t.round_reg.length.as_secs() as i64)
        {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting round length.");
                -1
            }
        }
    }

    /// Whether reg is open
    /// False on error
    #[no_mangle]
    pub extern "C" fn tid_reg_open(self) -> bool {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .tournament_query(self, |t| t.reg_open)
        {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting reg status.");
                false
            }
        }
    }

    /// Whether checkins are needed
    /// False on error
    #[no_mangle]
    pub extern "C" fn tid_require_check_in(self) -> bool {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .tournament_query(self, |t| t.require_check_in)
        {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting check in requirement.");
                false
            }
        }
    }

    /// Whether deck reg is needed
    /// False on error
    #[no_mangle]
    pub extern "C" fn tid_require_deck_reg(self) -> bool {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .tournament_query(self, |t| t.require_check_in)
        {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting deck reg requirement.");
                false
            }
        }
    }

    /// Returns the status
    /// Returns cancelled on error
    #[no_mangle]
    pub extern "C" fn tid_status(self) -> TournamentStatus {
        match SQUIRE_RUNTIME
            .get()
            .unwrap()
            .tournament_query(self, |t| t.status)
        {
            Ok(data) => data,
            Err(err) => {
                print_err(err, "getting tournament status.");
                TournamentStatus::Cancelled
            }
        }
    }

    /// Generates a round slip for a tournament
    /// NULL on error
    #[no_mangle]
    pub extern "C" fn tid_round_slips_html(self, __css: *const c_char) -> *const c_char {
        let mut ret: String = Default::default();
        SQUIRE_RUNTIME.get().unwrap().tournament_query(self, |t| {
            clone_string_c_string(&t.round_slips_html(unsafe { CStr::from_ptr(__css).to_str().unwrap() })
                .clone()
                .round_slips_html(unsafe { CStr::from_ptr(__css).to_str().unwrap() })
        });
        clone_string_to_c_string(ret.as_str())
    }

    // End of getters
    /// Closes a tournament removing it from the internal FFI state
    #[no_mangle]
    pub extern "C" fn close_tourn(self) -> bool {
        match SQUIRE_RUNTIME.get().unwrap().remove_tournament(self) {
            Some(_) => true,
            None => {
                println!("[FFI]: Cannot find tournament in clsoe_tourn");
                false
            }
        }
    }

    /// Saves a tournament to a name
    /// Returns true if successful, false if not.
    #[no_mangle]
    pub extern "C" fn save_tourn(self, __file: *const c_char) -> bool {
        let file = unsafe { CStr::from_ptr(__file).to_str().unwrap() };
        let file_backup = format!("{file}{BACKUP_EXT}");
        let _ = std::fs::remove_file(&file_backup);
        let _ = std::fs::rename(file, &file_backup);

        match SQUIRE_RUNTIME.get().unwrap().tournament_query(self, |t| {
            serde_json::to_string(&t).map(|json| std::fs::write(file, json))
        }) {
            Ok(Ok(Ok(()))) => true,
            Ok(Ok(Err(e))) => {
                println!("[FFI]: Tried to write to file {file} and got this error: {e}");
                false
            }
            Ok(Err(e)) => {
                println!("[FFI]: Tried to convert tournament to json and for this error: {e}");
                false
            }
            Err(err) => {
                print_err(err, "saving tournament to file.");
                false
            }
        }
    }
}

/// Loads a tournament from a file via serde
/// The tournament is then registered (stored on the heap)
/// CStr path to the tournament (alloc and, free on Cxx side)
/// Returns a NULL UUID (all 0s) if there is an error
#[no_mangle]
pub extern "C" fn load_tournament_from_file(__file: *const c_char) -> TournamentId {
    let file = unsafe { CStr::from_ptr(__file).to_str().unwrap() };
    let json = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(_) => {
            println!("[FFI]: Cannot read input file");
            return Uuid::default().into();
        }
    };

    let tournament: Tournament = match serde_json::from_str(&json) {
        Ok(t) => t,
        Err(_) => {
            println!("[FFI]: Input file is invalid");
            return Uuid::default().into();
        }
    };

    let rt = SQUIRE_RUNTIME.get().unwrap();

    if let Ok(()) = rt.tournament_query(tournament.id, |_| ()) {
        println!("[FFI]: Input tournament is already open");
        return Uuid::default().into();
    }

    let t_id = rt.create_tournament("TEMP".into(), TournamentPreset::Swiss, "TEMP".into());
    let _ = rt.mutate_tournament(t_id, |t| *t = tournament);
    t_id
}

/// Creates a tournament from the settings provided
/// Returns a NULL UUID (all 0s) if there is an error
#[no_mangle]
pub extern "C" fn new_tournament_from_settings(
    __file: *const c_char,
    __name: *const c_char,
    __format: *const c_char,
    preset: TournamentPreset,
    use_table_number: bool,
    game_size: u8,
    min_deck_count: u8,
    max_deck_count: u8,
    reg_open: bool,
    require_check_in: bool,
    require_deck_reg: bool,
) -> TournamentId {
    let name = String::from(unsafe { CStr::from_ptr(__name).to_str().unwrap().to_string() });
    let format = String::from(unsafe { CStr::from_ptr(__format).to_str().unwrap().to_string() });

    let rt = SQUIRE_RUNTIME.get().unwrap();
    let t_id = rt.create_tournament(name, preset, format);

    let _: () = rt
        .mutate_tournament(t_id, |t| {
            t.use_table_number = use_table_number;
            t.min_deck_count = min_deck_count;
            t.max_deck_count = max_deck_count;
            t.require_check_in = require_check_in;
            t.require_deck_reg = require_deck_reg;
            t.reg_open = reg_open;
            t.pairing_sys.match_size = game_size;
        })
        .unwrap();

    if !t_id.save_tourn(__file) {
        return Uuid::default().into();
    }

    t_id
}

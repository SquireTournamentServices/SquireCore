use std::{collections::HashMap, ffi::CStr, os::raw::c_char, time::Duration};

use serde_json;
use uuid::Uuid;

use crate::{
    accounts::SquireAccount,
    admin::Admin,
    ffi::{clone_string_to_c_string, FFI_TOURNAMENT_REGISTRY},
    identifiers::{AdminId, PlayerId, RoundId, TournamentId, UserAccountId},
    operations::{
        AdminOp,
        OpData::{Pair, RegisterPlayer},
        TournOp,
    },
    pairings::{PairingStyle, PairingSystem},
    players::PlayerRegistry,
    rounds::RoundRegistry,
    scoring::StandardScore,
    settings::{PairingSetting, TournamentSetting},
    tournament::{scoring_system_factory, Tournament, TournamentPreset, TournamentStatus},
};

use super::copy_to_system_pointer;

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
    pub extern "C" fn tid_standings(self: Self) -> *const PlayerScore<StandardScore> {
        let tourn = match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(t) => t,
            None => {
                return std::ptr::null();
            }
        };

        let scores = tourn.get_standings().scores;

        unsafe {
            copy_to_system_pointer(
                scores
                    .into_iter()
                    .map(|(pid, score)| PlayerScore { pid, score }),
            )
        }
    }

    /// Returns a raw pointer to players
    /// This is an array that is terminated by the NULL UUID
    /// This is heap allocted, please free it
    /// Returns NULL on error
    #[no_mangle]
    pub extern "C" fn tid_players(self: Self) -> *const PlayerId {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(t) => unsafe { copy_to_system_pointer(t.player_reg.players.keys().cloned()) },
            None => std::ptr::null(),
        }
    }

    /// Returns a raw pointer to rounds
    /// This is an array that is terminated by the NULL UUID
    /// This is heap allocted, please free it
    /// Returns NULL on error
    #[no_mangle]
    pub extern "C" fn tid_rounds(self: Self) -> *const RoundId {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(tourn) => unsafe {
                copy_to_system_pointer(tourn.round_reg.num_and_id.iter_right().cloned())
            },
            None => std::ptr::null(),
        }
    }

    /// Adds a player to a tournament
    /// On error a NULL UUID is returned
    #[no_mangle]
    pub extern "C" fn tid_add_player(self: Self, __name: *const c_char) -> PlayerId {
        let name = unsafe { CStr::from_ptr(__name).to_str().unwrap() };
        let op = TournOp::RegisterPlayer(SquireAccount::new(name.to_string(), name.to_string()));

        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get_mut(&self) {
            Some(mut t) => {
                match t.apply_op(op) {
                    Ok(RegisterPlayer(id)) => id,
                    Err(t_err) => {
                        println!("[FFI]: {t_err}");
                        Uuid::default().into()
                    }
                    // We are known the from that the data will take if it exists
                    // so we can ignore the other outcomes
                    _ => Uuid::default().into(),
                }
            }
            None => {
                println!("[FFI]: Cannot find tournament in tid_add_player");
                Uuid::default().into()
            }
        }
    }

    #[no_mangle]
    /// Drops a player for the tournament
    /// On error false is returned
    pub extern "C" fn tid_drop_player(self, pid: PlayerId, aid: AdminId) -> bool {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get_mut(&self) {
            Some(mut t) => {
                match t.apply_op(TournOp::AdminOp(aid, AdminOp::AdminDropPlayer(pid.into()))) {
                    Ok(_) => true,
                    Err(t_err) => {
                        println!("[FFI]: {t_err}");
                        false
                    }
                }
            }
            None => {
                println!("[FFI]: Cannot find tournament in tid_drop_player");
                false
            }
        }
    }

    /// Adds an admin to a local tournament in a way that is perfect for
    /// adding the system user.
    #[no_mangle]
    pub extern "C" fn tid_add_admin_local(
        self: Self,
        __name: *const c_char,
        aid: AdminId,
        uid: UserAccountId,
    ) -> bool {
        let name = unsafe { CStr::from_ptr(__name).to_str().unwrap() };
        let mut account = SquireAccount::new(name.to_string(), name.to_string());
        account.user_id = uid;

        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get_mut(&self) {
            Some(mut t) => {
                let admin = Admin::new(account);
                t.admins.insert(aid, admin.clone());
                println!("[FFI]: {} is now an admin.", aid);
                true
            }
            None => {
                println!("[FFI]: Cannot find tournament in tid_add_admin");
                false
            }
        }
    }

    /// Defrosts a tournament
    /// false on error, true on success.
    #[no_mangle]
    pub extern "C" fn tid_thaw(self: Self, aid: AdminId) -> bool {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get_mut(&self) {
            Some(mut t) => match t.apply_op(TournOp::AdminOp(aid, AdminOp::Thaw)) {
                Err(t_err) => {
                    println!("[FFI]: {t_err}");
                    false
                }
                Ok(_) => true,
            },
            None => {
                println!("[FFI]: Cannot find tournament in tid_thaw");
                false
            }
        }
    }

    /// Freezes a tournament
    /// false on error, true on success.
    #[no_mangle]
    pub extern "C" fn tid_freeze(self: Self, aid: AdminId) -> bool {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get_mut(&self) {
            Some(mut t) => match t.apply_op(TournOp::AdminOp(aid, AdminOp::Freeze)) {
                Err(t_err) => {
                    println!("[FFI]: {t_err}");
                    false
                }
                Ok(_) => true,
            },
            None => {
                println!("[FFI]: Cannot find tournament in tid_freeze");
                false
            }
        }
    }

    /// End a tournament
    /// false on error, true on success.
    #[no_mangle]
    pub extern "C" fn tid_end(self: Self, aid: AdminId) -> bool {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get_mut(&self) {
            Some(mut t) => match t.apply_op(TournOp::AdminOp(aid, AdminOp::End)) {
                Err(t_err) => {
                    println!("[FFI]: {t_err}");
                    false
                }
                Ok(_) => true,
            },
            None => {
                println!("[FFI]: Cannot find tournament in tid_end");
                false
            }
        }
    }

    /// Cancels a tournament
    /// false on error, true on success.
    #[no_mangle]
    pub extern "C" fn tid_cancel(self: Self, aid: AdminId) -> bool {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get_mut(&self) {
            Some(mut t) => match t.apply_op(TournOp::AdminOp(aid, AdminOp::Cancel)) {
                Err(t_err) => {
                    println!("[FFI]: {t_err}");
                    false
                }
                Ok(_) => true,
            },
            None => {
                println!("[FFI]: Cannot find tournament in tid_cancel");
                false
            }
        }
    }

    /// Starts a tournament
    /// false on error, true on success.
    #[no_mangle]
    pub extern "C" fn tid_start(self: Self, aid: AdminId) -> bool {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get_mut(&self) {
            Some(mut t) => match t.apply_op(TournOp::AdminOp(aid, AdminOp::Start)) {
                Err(t_err) => {
                    println!("[FFI]: {t_err}");
                    false
                }
                Ok(_) => true,
            },
            None => {
                println!("[FFI]: Cannot find tournament in tid_start");
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
        let mut tournament = match FFI_TOURNAMENT_REGISTRY.get().unwrap().get_mut(&self) {
            Some(t) => t,
            None => {
                println!("[FFI]: cannot find tournament in update settings");
                return false;
            }
        };

        // Sort input strings out
        let format =
            String::from(unsafe { CStr::from_ptr(__format).to_str().unwrap().to_string() });

        // Init list of operations to execute
        let mut op_vect: Vec<TournOp> = Vec::<TournOp>::new();
        if format != tournament.format {
            op_vect.push(TournOp::AdminOp(
                aid,
                AdminOp::UpdateTournSetting(TournamentSetting::Format(format)),
            ));
        }

        if starting_table_number != tournament.round_reg.starting_table {
            op_vect.push(TournOp::AdminOp(
                aid,
                AdminOp::UpdateTournSetting(TournamentSetting::StartingTableNumber(
                    starting_table_number,
                )),
            ));
        }

        if use_table_number != tournament.use_table_number {
            op_vect.push(TournOp::AdminOp(
                aid,
                AdminOp::UpdateTournSetting(TournamentSetting::UseTableNumbers(use_table_number)),
            ));
        }

        if game_size != tournament.pairing_sys.match_size {
            op_vect.push(TournOp::AdminOp(
                aid,
                AdminOp::UpdateTournSetting(TournamentSetting::PairingSetting(
                    PairingSetting::MatchSize(game_size),
                )),
            ));
        }

        let old_max_deck_count = tournament.max_deck_count;
        if min_deck_count != tournament.min_deck_count {
            op_vect.push(TournOp::AdminOp(
                aid,
                AdminOp::UpdateTournSetting(TournamentSetting::MinDeckCount(min_deck_count)),
            ));
        }

        if max_deck_count != tournament.max_deck_count {
            op_vect.push(TournOp::AdminOp(
                aid,
                AdminOp::UpdateTournSetting(TournamentSetting::MaxDeckCount(max_deck_count)),
            ));
        }

        // This is really annoying as if the new min deck count is above the
        // current max then it errors. If this is the case then add max deck
        // count first
        if min_deck_count > old_max_deck_count && op_vect.len() > 1 {
            let len = op_vect.len();
            op_vect.swap(len - 1, len - 2);
        }

        if match_length != tournament.round_reg.length.as_secs() {
            op_vect.push(TournOp::AdminOp(
                aid,
                AdminOp::UpdateTournSetting(TournamentSetting::RoundLength(Duration::new(
                    match_length,
                    0,
                ))),
            ));
        }

        if reg_open != tournament.reg_open {
            op_vect.push(TournOp::AdminOp(aid, AdminOp::UpdateReg(reg_open)));
        }

        if require_check_in != tournament.require_check_in {
            op_vect.push(TournOp::AdminOp(
                aid,
                AdminOp::UpdateTournSetting(TournamentSetting::RequireCheckIn(require_check_in)),
            ));
        }

        if require_deck_reg != tournament.require_deck_reg {
            op_vect.push(TournOp::AdminOp(
                aid,
                AdminOp::UpdateTournSetting(TournamentSetting::RequireDeckReg(require_deck_reg)),
            ));
        }

        // Apply all settings
        let opt_err = op_vect
            .into_iter()
            .map(|op| tournament.apply_op(op))
            .find(|res| res.is_err());
        if let Some(Err(t_err)) = &opt_err {
            println!("[FFI]: update_settings error: {t_err}");
        }
        opt_err.is_some()
    }

    /// Pairs a set of rounds
    /// Returns a null terminated list of the round ids
    /// Returns NULL on error
    #[no_mangle]
    pub extern "C" fn tid_pair_round(self: Self, aid: AdminId) -> *const RoundId {
        let op = TournOp::AdminOp(aid, AdminOp::PairRound);
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get_mut(&self) {
            Some(mut t) => match t.apply_op(op) {
                Ok(Pair(ident_vec)) => unsafe { copy_to_system_pointer(ident_vec.into_iter()) },
                Err(t_err) => {
                    println!("[FFI]: {t_err}");
                    std::ptr::null()
                }
                Ok(_) => {
                    println!("[FFI]: Error in tid_pair_round");
                    std::ptr::null()
                }
            },
            None => {
                println!("[FFI]: Cannot find tournament");
                std::ptr::null()
            }
        }
    }

    /// Returns the name of a tournament
    /// Returns NULL if an error happens
    /// This is heap allocated, please free it
    #[no_mangle]
    pub extern "C" fn tid_name(self: Self) -> *const c_char {
        FFI_TOURNAMENT_REGISTRY
            .get()
            .unwrap()
            .get(&self)
            .map(|t| clone_string_to_c_string(&t.name))
            .unwrap_or_else(|| {
                println!("[FFI]: Cannot find tournament");
                std::ptr::null::<i8>() as *mut i8
            })
    }

    /// Returns the format of a tournament
    /// Returns NULL if an error happens
    /// This is heap allocated, please free it
    #[no_mangle]
    pub extern "C" fn tid_format(self: Self) -> *const c_char {
        FFI_TOURNAMENT_REGISTRY
            .get()
            .unwrap()
            .get(&self)
            .map(|t| clone_string_to_c_string(&t.format))
            .unwrap_or_else(|| {
                println!("[FFI]: Cannot find tournament");
                std::ptr::null::<i8>() as *mut i8
            })
    }

    /// Returns the starting table number
    /// Retruns -1 on error
    #[no_mangle]
    pub extern "C" fn tid_starting_table_number(self: Self) -> i32 {
        FFI_TOURNAMENT_REGISTRY
            .get()
            .unwrap()
            .get(&self)
            .map(|t| t.round_reg.starting_table as i32)
            .unwrap_or_else(|| {
                println!("[FFI]: Cannot find tournament");
                -1
            })
    }

    /// Returns whether table numbers are being used for this tournament
    /// false, is the error value (kinda sketchy)
    #[no_mangle]
    pub extern "C" fn tid_use_table_number(self: Self) -> bool {
        FFI_TOURNAMENT_REGISTRY
            .get()
            .unwrap()
            .get(&self)
            .map(|t| t.use_table_number)
            .unwrap_or_else(|| {
                println!("Cannot find tournament in tourn_id.use_table_number();");
                false
            })
    }

    /// Returns the game size
    /// -1 is the error value
    #[no_mangle]
    pub extern "C" fn tid_game_size(self: Self) -> i32 {
        FFI_TOURNAMENT_REGISTRY
            .get()
            .unwrap()
            .get(&self)
            .map(|t| t.pairing_sys.match_size as i32)
            .unwrap_or_else(|| {
                println!("Cannot find tournament in tourn_id.game_size();");
                -1
            })
    }

    /// Returns the min deck count
    /// -1 is the error value
    #[no_mangle]
    pub extern "C" fn tid_min_deck_count(self: Self) -> i32 {
        FFI_TOURNAMENT_REGISTRY
            .get()
            .unwrap()
            .get(&self)
            .map(|t| t.min_deck_count as i32)
            .unwrap_or_else(|| {
                println!("Cannot find tournament in tourn_id.min_deck_count();");
                -1
            })
    }

    /// Returns the max deck count
    /// -1 is the error value
    #[no_mangle]
    pub extern "C" fn tid_max_deck_count(self: Self) -> i32 {
        FFI_TOURNAMENT_REGISTRY
            .get()
            .unwrap()
            .get(&self)
            .map(|t| t.max_deck_count as i32)
            .unwrap_or_else(|| {
                println!("Cannot find tournament in tourn_id.max_deck_count();");
                -1
            })
    }

    /// Returns the pairing type
    /// This is of type TournamentPreset, but an int to let me return error values
    /// -1 is error value
    #[no_mangle]
    pub extern "C" fn tid_pairing_type(self: Self) -> i32 {
        use TournamentPreset::*;
        FFI_TOURNAMENT_REGISTRY
            .get()
            .unwrap()
            .get(&self)
            .map(|t| match t.pairing_sys.style {
                PairingStyle::Swiss(_) => Swiss as i32,
                PairingStyle::Fluid(_) => Fluid as i32,
            })
            .unwrap_or_else(|| {
                println!("Cannot find tournament in tourn_id.pairing_type();");
                -1
            })
    }

    /// Returns the round length
    /// -1 on error
    #[no_mangle]
    pub extern "C" fn tid_round_length(self: Self) -> i64 {
        FFI_TOURNAMENT_REGISTRY
            .get()
            .unwrap()
            .get(&self)
            .map(|t| t.round_reg.length.as_secs() as i64)
            .unwrap_or_else(|| {
                println!("Cannot find tournament in tourn_id.round_length();");
                -1
            })
    }

    /// Whether reg is open
    /// False on error
    #[no_mangle]
    pub extern "C" fn tid_reg_open(self: Self) -> bool {
        FFI_TOURNAMENT_REGISTRY
            .get()
            .unwrap()
            .get(&self)
            .map(|t| t.reg_open)
            .unwrap_or_else(|| {
                println!("Cannot find tournament in tourn_id.reg_open();");
                false
            })
    }

    /// Whether checkins are needed
    /// False on error
    #[no_mangle]
    pub extern "C" fn tid_require_check_in(self: Self) -> bool {
        FFI_TOURNAMENT_REGISTRY
            .get()
            .unwrap()
            .get(&self)
            .map(|t| t.require_deck_reg)
            .unwrap_or_else(|| {
                println!("Cannot find tournament in tourn_id.require_check_in();");
                false
            })
    }

    /// Whether deck reg is needed
    /// False on error
    #[no_mangle]
    pub extern "C" fn tid_require_deck_reg(self: Self) -> bool {
        FFI_TOURNAMENT_REGISTRY
            .get()
            .unwrap()
            .get(&self)
            .map(|t| t.require_deck_reg)
            .unwrap_or_else(|| {
                println!("Cannot find tournament in tourn_id.require_deck_reg();");
                false
            })
    }

    /// Returns the status
    /// Returns cancelled on error
    #[no_mangle]
    pub extern "C" fn tid_status(self: Self) -> TournamentStatus {
        FFI_TOURNAMENT_REGISTRY
            .get()
            .unwrap()
            .get(&self)
            .map(|t| t.status)
            .unwrap_or_else(|| {
                println!("Cannot find tournament in tourn_id.status();");
                TournamentStatus::Cancelled
            })
    }

    // End of getters

    /// Closes a tournament removing it from the internal FFI state
    #[no_mangle]
    pub extern "C" fn close_tourn(self: Self) -> bool {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().remove(&self) {
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
    pub extern "C" fn save_tourn(self: Self, __file: *const c_char) -> bool {
        let tournament = match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(t) => t,
            None => {
                println!("[FFI]: Cannot find tournament in save_tourn");
                return false;
            }
        };

        let json = match serde_json::to_string(&tournament.value()) {
            Ok(v) => v,
            Err(e) => {
                println!("[FFI]: Cannot convert tournament to json in save_tourn: {e}");
                return false;
            }
        };

        // Backup old data, do check for errors.
        let file = unsafe { CStr::from_ptr(__file).to_str().unwrap() };
        let file_backup = format!("{file}{BACKUP_EXT}");
        let _ = std::fs::remove_file(&file_backup);
        let _ = std::fs::rename(file, &file_backup);

        match std::fs::write(file, json) {
            Ok(_) => true,
            Err(e) => {
                println!("[FFI]: Cannot write file: {}", e);
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
        Ok(s) => s.to_string(),
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

    // Cannot open the same tournament twice
    FFI_TOURNAMENT_REGISTRY
        .get()
        .unwrap()
        .get(&tournament.id)
        .map(|_| {
            println!("[FFI]: Input tournament is already open");
            Uuid::default().into()
        })
        .unwrap_or_else(|| {
            let tid = tournament.id;
            FFI_TOURNAMENT_REGISTRY
                .get()
                .unwrap()
                .insert(tid, tournament);
            tid
        })
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
    let mut tournament: Tournament = Tournament {
        id: TournamentId::new(Uuid::new_v4()),
        use_table_number,
        min_deck_count,
        max_deck_count,
        require_check_in,
        require_deck_reg,
        reg_open,
        name: String::from(unsafe { CStr::from_ptr(__name).to_str().unwrap().to_string() }),
        format: String::from(unsafe { CStr::from_ptr(__format).to_str().unwrap().to_string() }),
        player_reg: PlayerRegistry::new(),
        round_reg: RoundRegistry::new(0, Duration::from_secs(3000)),
        pairing_sys: PairingSystem::new(preset),
        scoring_sys: scoring_system_factory(preset),
        status: TournamentStatus::Planned,
        judges: HashMap::new(),
        admins: HashMap::new(),
    };

    tournament.pairing_sys.match_size = game_size;

    FFI_TOURNAMENT_REGISTRY
        .get()
        .unwrap()
        .insert(tournament.id, tournament.clone());

    if !tournament.id.save_tourn(__file) {
        return Uuid::default().into();
    }

    tournament.id
}

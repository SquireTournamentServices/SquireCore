use std::{
    alloc::{Allocator, Layout, System},
    collections::HashMap,
    ffi::CStr,
    os::raw::c_char,
    ptr,
    time::Duration,
};

use serde_json;
use uuid::Uuid;

use crate::{
    accounts::SquireAccount,
    ffi::{clone_string_to_c_string, FFI_TOURNAMENT_REGISTRY},
    identifiers::{PlayerId, PlayerIdentifier, TournamentId},
    operations::{OpData::RegisterPlayer, TournOp},
    pairings::{PairingStyle, PairingSystem},
    player_registry::PlayerRegistry,
    round_registry::RoundRegistry,
    tournament::{scoring_system_factory, Tournament, TournamentPreset, TournamentStatus},
};

const BACKUP_EXT: &str = ".bak";

/// TournamentIds can be used to get data safely from
/// the Rust lib with these methods
impl TournamentId {
    /// Returns a raw pointer to players
    /// This is an array that is terminated by the NULL UUID
    /// This is heap allocted, please free it
    /// Returns NULL on error
    #[no_mangle]
    pub extern "C" fn tid_players(self: Self) -> *const PlayerId {
        let tourn = match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(t) => t,
            None => {
                return std::ptr::null();
            }
        };

        let mut players = tourn.player_reg.get_player_ids();
        players.push(Uuid::default().into());

        let len = players.len() * std::mem::size_of::<PlayerId>();

        let ptr = System
            .allocate(Layout::from_size_align(len, 1).unwrap())
            .unwrap()
            .as_mut_ptr() as *mut PlayerId;

        let slice = unsafe { &mut *(ptr::slice_from_raw_parts(ptr, len) as *mut [PlayerId]) };

        slice.iter_mut().zip(players.iter()).for_each(|(dst, p)| {
            *dst = *p;
        });

        ptr
    }

    /// Adds a player to a tournament
    /// On error a NULL UUID is returned
    #[no_mangle]
    pub unsafe extern "C" fn tid_add_player(self: Self, __name: *const c_char) -> PlayerId {
        let name: &str = CStr::from_ptr(__name).to_str().unwrap();
        let op: TournOp =
            TournOp::RegisterPlayer(SquireAccount::new(name.to_string(), name.to_string()));

        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get_mut(&self) {
            Some(mut t) => {
                match t.apply_op(op) {
                    Ok(RegisterPlayer(PlayerIdentifier::Id(id))) => id,
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

    /// Returns the name of a tournament
    /// Returns NULL if an error happens
    /// This is heap allocated, please free it
    #[no_mangle]
    pub unsafe extern "C" fn tid_name(self: Self) -> *const c_char {
        FFI_TOURNAMENT_REGISTRY
            .get()
            .unwrap()
            .get(&self)
            .map(|t| clone_string_to_c_string(t.clone().name))
            .unwrap_or_else(|| {
                println!("[FFI]: Cannot find tournament");
                std::ptr::null::<i8>() as *mut i8
            })
    }

    /// Returns the format of a tournament
    /// Returns NULL if an error happens
    /// This is heap allocated, please free it
    #[no_mangle]
    pub unsafe extern "C" fn tid_format(self: Self) -> *const c_char {
        FFI_TOURNAMENT_REGISTRY
            .get()
            .unwrap()
            .get(&self)
            .map(|t| clone_string_to_c_string(t.format.clone()))
            .unwrap_or_else(|| {
                println!("[FFI]: Cannot find tournament");
                std::ptr::null::<i8>() as *mut i8
            })
    }

    /// Returns whether table numbers are being used for this tournament
    /// false, is the error value (kinda sketchy)
    #[no_mangle]
    pub unsafe extern "C" fn tid_use_table_number(self: Self) -> bool {
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
    pub unsafe extern "C" fn tid_game_size(self: Self) -> i32 {
        FFI_TOURNAMENT_REGISTRY
            .get()
            .unwrap()
            .get(&self)
            .map(|t| t.game_size as i32)
            .unwrap_or_else(|| {
                println!("Cannot find tournament in tourn_id.game_size();");
                -1
            })
    }

    /// Returns the min deck count
    /// -1 is the error value
    #[no_mangle]
    pub unsafe extern "C" fn tid_min_deck_count(self: Self) -> i32 {
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
    pub unsafe extern "C" fn tid_max_deck_count(self: Self) -> i32 {
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

    /// Whether reg is open
    /// False on error
    #[no_mangle]
    pub unsafe extern "C" fn tid_reg_open(self: Self) -> bool {
        FFI_TOURNAMENT_REGISTRY
            .get()
            .unwrap()
            .get(&self)
            .map(|t| t.reg_open)
            .unwrap_or_else(|| {
                println!("Cannot find tournament in tourn_id.reg_open();");
                return false;
            })
    }

    /// Whether checkins are needed
    /// False on error
    #[no_mangle]
    pub unsafe extern "C" fn tid_require_check_in(self: Self) -> bool {
        FFI_TOURNAMENT_REGISTRY
            .get()
            .unwrap()
            .get(&self)
            .map(|t| t.require_deck_reg)
            .unwrap_or_else(|| {
                println!("Cannot find tournament in tourn_id.require_check_in();");
                return false;
            })
    }

    /// Whether deck reg is needed
    /// False on error
    #[no_mangle]
    pub unsafe extern "C" fn tid_require_deck_reg(self: Self) -> bool {
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
    pub unsafe extern "C" fn tid_status(self: Self) -> TournamentStatus {
        FFI_TOURNAMENT_REGISTRY
            .get()
            .unwrap()
            .get(&self)
            .map(|t| t.status)
            .unwrap_or_else(|| {
                println!("Cannot find tournament in tourn_id.status();");
                return TournamentStatus::Cancelled;
            })
    }

    // End of getters

    /// Closes a tournament removing it from the internal FFI state
    #[no_mangle]
    pub unsafe extern "C" fn close_tourn(self: Self) -> bool {
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
        std::fs::remove_file(file_backup.clone());
        std::fs::rename(file, file_backup.clone());

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
    let file: &str = unsafe { CStr::from_ptr(__file).to_str().unwrap() };
    let json: String = match std::fs::read_to_string(file) {
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
    let tournament: Tournament = Tournament {
        id: TournamentId::new(Uuid::new_v4()),
        name: String::from(unsafe { CStr::from_ptr(__name).to_str().unwrap().to_string() }),
        use_table_number,
        format: String::from(unsafe { CStr::from_ptr(__format).to_str().unwrap().to_string() }),
        game_size,
        min_deck_count,
        max_deck_count,
        player_reg: PlayerRegistry::new(),
        round_reg: RoundRegistry::new(0, Duration::from_secs(3000)),
        pairing_sys: PairingSystem::new(preset),
        scoring_sys: scoring_system_factory(preset),
        reg_open,
        require_check_in,
        require_deck_reg,
        status: TournamentStatus::Planned,
        judges: HashMap::new(),
        admins: HashMap::new(),
    };

    FFI_TOURNAMENT_REGISTRY
        .get()
        .unwrap()
        .insert(tournament.id, tournament.clone());

    if !tournament.id.save_tourn(__file) {
        return Uuid::default().into();
    }
    tournament.id
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
pub extern "C" tid_update_settings(
    tid: TournamentId,
    __format: *const c_char,
    starting_table_number: u64,
    use_table_number: bool,
    game_size: u8,
    min_deck_count: u8,
    max_deck_count: u8,
    reg_open: bool,
    require_check_in: bool,
    require_deck_reg: bool) -> bool {
    let tournament: Tournament;
    unsafe {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(v) => tournament = v.value().clone(),
            None => {
                println!("[FFI]: Cannot find tournament in update_settings");
                return false;
            }
        }
    }

    // Sort input strings out
    let format: String::from(unsafe { CStr::from_ptr(__format).to_str().unwrap().to_string() });

    // Init list of operations to execute
    let mut op_vect: Vec<TournOp> = Vec::<TournOp>::new();
    if format != tournament.format {
        op_vect.push(TournOp::UpdateSetting(tid, TournamentSetting::Format(format)));
    }

    if starting_table_number != tournament.round_reg.starting_table {
        op_vect.push(TournOp::UpdateSetting(tid, TournamentSetting::StartingTableNumber(starting_table_number)));
    }

    if use_table_number != tournament.use_table_number {
        op_vect.push(TournOp::UpdateSetting(tid, TournamentSetting::UseTableNumbers(use_table_number)));
    }

    if game_size != tournament.game_size {
        todo!("fix me");
        //op_vect.push(TournOp::UpdateSetting(tid, TournamentSetting::PairingSetting::MatchSize(game_size)));
    }

    min_deck_count: u8,
    max_deck_count: u8,
    reg_open: bool,
    require_check_in: bool,
    require_deck_reg: bool) -> bool {
    // Apply all settings, rollback on error.
    
    // Panic on double trouble
}

use crate::accounts::SquireAccount;
use crate::ffi::{clone_string_to_c_string, FFI_TOURNAMENT_REGISTRY};
use crate::operations::OpData::RegisterPlayer;
use crate::operations::TournOp;
use crate::round_registry::RoundRegistry;
use crate::tournament::pairing_system_factory;
use crate::tournament::scoring_system_factory;
use crate::tournament::PairingSystem::{Fluid, Swiss};
use crate::tournament::{Tournament, TournamentPreset, TournamentStatus};
use crate::{
    identifiers::{PlayerId, PlayerIdentifier, TournamentId},
    player_registry::PlayerRegistry,
};
use serde_json;
use std::alloc::{Allocator, Layout, System};
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;
use std::time::Duration;
use std::vec::Vec;
use uuid::Uuid;
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
        unsafe {
            let tourn: Tournament;
            match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
                Some(t) => tourn = t.value().clone(),
                None => {
                    return std::ptr::null();
                }
            }

            let players: Vec<PlayerId> = tourn.player_reg.get_player_ids();

            let len: usize = (players.len() + 1) * std::mem::size_of::<PlayerId>();

            let ptr = System
                .allocate(Layout::from_size_align(len, 1).unwrap())
                .unwrap()
                .as_mut_ptr() as *mut PlayerId;
            let slice = &mut *(ptr::slice_from_raw_parts(ptr, len) as *mut [PlayerId]);
            let mut i: usize = 0;
            while i < players.len() {
                slice[i] = players[i];
                i += 1;
            }
            slice[i] = Uuid::default().into();
            return ptr;
        }
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
                    Ok(RegisterPlayer(PlayerIdentifier::Id(id))) => {
                        return id;
                    }
                    Err(t_err) => {
                        println!("[FFI]: {t_err}");
                        return Uuid::default().into();
                    }
                    // We are known the from that the data will take if it exists
                    // so we can ignore the other outcomes
                    _ => {
                        return Uuid::default().into();
                    }
                };
            }
            None => {
                println!("[FFI]: Cannot find tournament in tid_add_player");
                return Uuid::default().into();
            }
        }
    }

    /// Returns the name of a tournament
    /// Returns NULL if an error happens
    /// This is heap allocated, please free it
    #[no_mangle]
    pub unsafe extern "C" fn tid_name(self: Self) -> *const c_char {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(t) => {
                return clone_string_to_c_string(t.clone().name);
            }
            None => {
                println!("[FFI]: Cannot find tournament");
                return std::ptr::null();
            }
        }
    }

    /// Returns the format of a tournament
    /// Returns NULL if an error happens
    /// This is heap allocated, please free it
    #[no_mangle]
    pub unsafe extern "C" fn tid_format(self: Self) -> *const c_char {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(t) => {
                return clone_string_to_c_string(t.clone().format);
            }
            None => {
                println!("[FFI]: Cannot find tournament");
                return std::ptr::null();
            }
        }
    }

    /// Returns whether table numbers are being used for this tournament
    /// false, is the error value (kinda sketchy)
    #[no_mangle]
    pub unsafe extern "C" fn tid_use_table_number(self: Self) -> bool {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(t) => return t.value().use_table_number,
            None => {
                println!("Cannot find tournament in tourn_id.use_table_number();");
                return false;
            }
        }
    }

    /// Returns the game size
    /// -1 is the error value
    #[no_mangle]
    pub unsafe extern "C" fn tid_game_size(self: Self) -> i32 {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(t) => return t.value().game_size as i32,
            None => {
                println!("Cannot find tournament in tourn_id.game_size();");
                return -1;
            }
        }
    }

    /// Returns the min deck count
    /// -1 is the error value
    #[no_mangle]
    pub unsafe extern "C" fn tid_min_deck_count(self: Self) -> i32 {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(t) => return t.value().min_deck_count as i32,
            None => {
                println!("Cannot find tournament in tourn_id.min_deck_count();");
                return -1;
            }
        }
    }

    /// Returns the max deck count
    /// -1 is the error value
    #[no_mangle]
    pub unsafe extern "C" fn tid_max_deck_count(self: Self) -> i32 {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(t) => return t.value().max_deck_count as i32,
            None => {
                println!("Cannot find tournament in tourn_id.max_deck_count();");
                return -1;
            }
        }
    }

    /// Returns the pairing type
    /// This is of type TournamentPreset, but an int to let me return error values
    /// -1 is error value
    #[no_mangle]
    pub extern "C" fn tid_pairing_type(self: Self) -> i32 {
        let tourn: Tournament;
        unsafe {
            match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
                Some(t) => tourn = t.value().clone(),
                None => {
                    println!("Cannot find tournament in tourn_id.pairing_type();");
                    return -1;
                }
            }
        }

        match tourn.pairing_sys {
            Swiss(_) => {
                return TournamentPreset::Swiss as i32;
            }
            Fluid(_) => {
                return TournamentPreset::Fluid as i32;
            }
        }
    }

    /// Whether reg is open
    /// False on error
    #[no_mangle]
    pub unsafe extern "C" fn tid_reg_open(self: Self) -> bool {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(t) => return t.reg_open,
            None => {
                println!("Cannot find tournament in tourn_id.reg_open();");
                return false;
            }
        }
    }

    /// Whether checkins are needed
    /// False on error
    #[no_mangle]
    pub unsafe extern "C" fn tid_require_check_in(self: Self) -> bool {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(t) => return t.require_check_in,
            None => {
                println!("Cannot find tournament in tourn_id.require_check_in();");
                return false;
            }
        }
    }

    /// Whether deck reg is needed
    /// False on error
    #[no_mangle]
    pub unsafe extern "C" fn tid_require_deck_reg(self: Self) -> bool {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(t) => return t.require_deck_reg,
            None => {
                println!("Cannot find tournament in tourn_id.require_deck_reg();");
                return false;
            }
        }
    }

    /// Returns the status
    /// Returns cancelled on error
    #[no_mangle]
    pub unsafe extern "C" fn tid_status(self: Self) -> TournamentStatus {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(t) => return t.status,
            None => {
                println!("Cannot find tournament in tourn_id.status();");
                return TournamentStatus::Cancelled;
            }
        }
    }

    // End of getters

    /// Closes a tournament removing it from the internal FFI state
    #[no_mangle]
    pub unsafe extern "C" fn close_tourn(self: Self) -> bool {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().remove(&self) {
            Some(_) => {
                return true;
            }
            None => {
                println!("[FFI]: Cannot find tournament in clsoe_tourn");
                return false;
            }
        }
    }

    /// Saves a tournament to a name
    /// Returns true if successful, false if not.
    #[no_mangle]
    pub extern "C" fn save_tourn(self: Self, __file: *const c_char) -> bool {
        let file: &str = unsafe { CStr::from_ptr(__file).to_str().unwrap() };
        let tournament: Tournament;
        unsafe {
            match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
                Some(v) => tournament = v.value().clone(),
                None => {
                    println!("[FFI]: Cannot find tournament in save_tourn");
                    return false;
                }
            }
        }

        let json: String;
        match serde_json::to_string::<Tournament>(&tournament) {
            Ok(v) => json = v,
            Err(e) => {
                println!(
                    "[FFI]: Cannot convert tournament to json in save_tourn: {}",
                    e
                );
                return false;
            }
        }

        // Backup old data, do check for errors.
        let file_backup: String = file.to_string() + &BACKUP_EXT.to_string();
        std::fs::remove_file(file_backup.clone());
        std::fs::rename(file, file_backup.clone());

        match std::fs::write(file, json) {
            Ok(_) => {
                return true;
            }
            Err(e) => {
                println!("[FFI]: Cannot write file: {}", e);
                return false;
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
    let json: String;
    match std::fs::read_to_string(file) {
        Ok(v) => json = v.to_string(),
        Err(_) => {
            println!("[FFI]: Cannot read input file");
            return Uuid::default().into();
        }
    };

    let tournament: Tournament;
    match serde_json::from_str::<Tournament>(&json) {
        Ok(v) => tournament = v,
        Err(_) => {
            println!("[FFI]: Input file is invalid");
            return Uuid::default().into();
        }
    };

    // Cannot open the same tournament twice
    unsafe {
        if FFI_TOURNAMENT_REGISTRY
            .get()
            .unwrap()
            .contains_key(&tournament.id)
        {
            println!("[FFI]: Input tournament is already open");
            return Uuid::default().into();
        }

        let tid: TournamentId = tournament.id.clone();
        FFI_TOURNAMENT_REGISTRY
            .get()
            .unwrap()
            .insert(tid, tournament);
        return tid;
    }
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
        use_table_number: use_table_number,
        format: String::from(unsafe { CStr::from_ptr(__format).to_str().unwrap().to_string() }),
        game_size: game_size,
        min_deck_count: min_deck_count,
        max_deck_count: max_deck_count,
        player_reg: PlayerRegistry::new(),
        round_reg: RoundRegistry::new(0, Duration::from_secs(3000)),
        pairing_sys: pairing_system_factory(&preset, 2),
        scoring_sys: scoring_system_factory(&preset),
        reg_open: reg_open,
        require_check_in: require_check_in,
        require_deck_reg: require_deck_reg,
        status: TournamentStatus::Planned,
        judges: HashMap::new(),
        admins: HashMap::new(),
    };
    let tid: TournamentId = tournament.id;

    unsafe {
        FFI_TOURNAMENT_REGISTRY
            .get_mut()
            .unwrap()
            .insert(tid, tournament.clone());
    }

    if !tournament.id.save_tourn(__file) {
        return Uuid::default().into();
    }
    return tournament.id;
}

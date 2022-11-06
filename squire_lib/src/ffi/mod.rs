use std::{
    alloc::{Allocator, Layout, System},
    borrow::Cow,
    os::raw::{c_char, c_void},
    ptr,
};

use chrono::Utc;
use dashmap::DashMap;
use once_cell::sync::OnceCell;
use crate::{
    error::TournamentError,
    identifiers::{PlayerId, RoundId, TournamentId},
    operations::{OpData, TournOp},
    players::Player,
    rounds::Round,
    tournament::{Tournament, TournamentPreset},
};

/// Contains the ffi C bindings for players used in SquireDesktop
pub mod ffi_player;
/// Contains the ffi C bindings for a tournament used in SquireDesktop
pub mod ffi_rounds;
/// Contains the ffi C bindings for a tournament used in SquireDesktop
pub mod ffi_tournament;

/// A map of tournament ids to tournaments
/// this is used for allocating ffi tournaments
/// all ffi tournaments are always deeply copied
/// at the lanuage barrier
pub static FFI_TOURNAMENT_REGISTRY: OnceCell<DashMap<TournamentId, Tournament>> = OnceCell::new();

/// The runtime that contains everything needed to manage and query the tournament model.
pub static SQUIRE_RUNTIME: OnceCell<SquireRuntime> = OnceCell::new();

/// The struct that contains everything needed to manage and query the tournament model.
#[derive(Debug, Default)]
pub struct SquireRuntime {
    tourns: DashMap<TournamentId, Tournament>,
}

/// Call this in main()
/// Inits the internal structs of squire lib for FFI.
#[no_mangle]
pub extern "C" fn init_squire_ffi() {
    SQUIRE_RUNTIME.set(SquireRuntime::default()).unwrap();
}

/// Takes an iterator to some data, allocates it to a slice, and returns a pointer to the start of
/// that slice. This method to primarily used to pass a collection of data from the Rust side to
/// the C++ side of the FFI boundary.
///
/// Safety check: To safely call this function you must ensure two things
///  1) `T::default()` is the null representation of `T`, i.e. `0x0` as the final element of the
///     slice must be null.
///  2) `T` must be safe to pass across the language boundary
pub unsafe fn copy_to_system_pointer<T, I>(iter: I) -> *const T
where
    T: Default,
    I: ExactSizeIterator<Item = T>,
{
    let length = iter.len();
    let len = (length + 1) * std::mem::size_of::<T>();
    let ptr = System
        .allocate(Layout::from_size_align(len, 1).unwrap())
        .unwrap()
        .as_mut_ptr() as *mut T;
    let slice = &mut *(ptr::slice_from_raw_parts(ptr, len) as *mut [T]);
    slice.iter_mut().zip(iter).for_each(|(dst, p)| {
        *dst = p;
    });
    slice[length] = T::default();
    ptr
}

/// Helper function for cloning strings. Assumes that the given string is a Rust string, i.e. it
/// does not end in a NULL char. Returns NULL on error
pub fn clone_string_to_c_string(s: &str) -> *const c_char {
    let ptr = System
        .allocate(Layout::from_size_align(s.len() + 1, 1).unwrap())
        .unwrap()
        .as_mut_ptr() as *mut c_char;

    let slice = unsafe { &mut *(ptr::slice_from_raw_parts(ptr, s.len() + 1) as *mut [c_char]) };
    slice.iter_mut().zip(s.chars()).for_each(|(dst, c)| {
        *dst = c as i8;
    });

    slice[s.len()] = char::default() as i8;

    ptr
}

/// Deallocates a block assigned in the FFI portion,
/// use this when handling with squire strings
#[no_mangle]
pub extern "C" fn sq_free(pointer: *mut c_void, len: usize) {
    unsafe {
        System.deallocate(
            ptr::NonNull::new(pointer as *mut u8).unwrap(),
            Layout::from_size_align(len, 1).unwrap(),
        );
    }
}

/// The enum that encodes what could go wrong while performing an action
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionError {
    /// The given tournament could not be found
    TournamentNotFound(TournamentId),
    /// The given round could not be found
    RoundNotFound(TournamentId, RoundId),
    /// The given player could not be found
    PlayerNotFound(TournamentId, PlayerId),
    /// An wrap for a tournament error
    OperationError(TournamentId, TournamentError),
}

impl SquireRuntime {
    /// Looks up a tournament and performs the given tournament operation
    pub fn apply_operation(&self, t_id: TournamentId, op: TournOp) -> Result<OpData, ActionError> {
        self.tourns
            .get_mut(&t_id)
            .ok_or_else(|| ActionError::TournamentNotFound(t_id))?
            .apply_op(Utc::now(), op)
            .map_err(|err| ActionError::OperationError(t_id, err))
    }

    /// Creates a tournament, stores it in the runtime, and returns its id
    pub fn create_tournament(
        &self,
        name: String,
        preset: TournamentPreset,
        format: String,
    ) -> TournamentId {
        let tourn = Tournament::from_preset(name, preset, format);
        let id = tourn.id;
        self.tourns.insert(id, tourn);
        id
    }

    /// Removes a tournament from the runtime and returns it, if found
    pub fn remove_tournament(&self, t_id: TournamentId) -> Option<Tournament> {
        self.tourns.remove(&t_id).map(|(_, t)| t)
    }

    /// Looks up a tournament and performs the given tournament operation
    pub fn mutate_tournament<OP, OUT>(&self, t_id: TournamentId, op: OP) -> Result<OUT, ActionError>
    where
        OP: FnOnce(&mut Tournament) -> OUT,
    {
        self.tourns
            .get_mut(&t_id)
            .map(|mut t| (op)(&mut t))
            .ok_or_else(|| ActionError::TournamentNotFound(t_id))
    }

    /// Looks up a tournament and performs the given query
    pub fn tournament_query<Q, O>(&self, t_id: TournamentId, query: Q) -> Result<O, ActionError>
    where
        Q: FnOnce(&Tournament) -> O,
    {
        self.tourns
            .get(&t_id)
            .map(|t| (query)(&t))
            .ok_or_else(|| ActionError::TournamentNotFound(t_id))
    }

    /// Looks up a player and performs the given query
    pub fn round_query<Q, O>(
        &self,
        t_id: TournamentId,
        r_id: RoundId,
        query: Q,
    ) -> Result<O, ActionError>
    where
        Q: FnOnce(&Round) -> O,
    {
        self.tourns
            .get(&t_id)
            .ok_or_else(|| ActionError::TournamentNotFound(t_id))?
            .get_round_by_id(&r_id)
            .map(query)
            .map_err(|_| ActionError::RoundNotFound(t_id, r_id))
    }

    /// Looks up a player and performs the given query
    pub fn player_query<Q, O>(
        &self,
        t_id: TournamentId,
        p_id: PlayerId,
        query: Q,
    ) -> Result<O, ActionError>
    where
        Q: FnOnce(&Player) -> O,
    {
        self.tourns
            .get(&t_id)
            .ok_or_else(|| ActionError::TournamentNotFound(t_id))?
            .get_player_by_id(&p_id)
            .map(query)
            .map_err(|_| ActionError::PlayerNotFound(t_id, p_id))
    }
}

/// Prints an error for debugging
pub fn print_err(err: ActionError, context: &str) {
    use ActionError::*;
    match err {
        TournamentNotFound(t_id) => {
            println!("[FFI]: Cannot find tournament '{t_id}' while {context}");
        }
        RoundNotFound(t_id, r_id) => {
            println!("[FFI]: Cannot find round '{r_id}' in tournament '{t_id}' while {context}");
        }
        PlayerNotFound(t_id, p_id) => {
            println!("[FFI]: Cannot find player '{p_id}' in tournament '{t_id}' while {context}");
        }
        OperationError(t_id, err) => {
            use TournamentError::*;
            let content = match err {
                IncorrectStatus(status) => {
                    Cow::Owned(format!("Incorrect tournament status '{status}'"))
                }
                PlayerLookup => Cow::Borrowed("Could not find player"),
                RoundLookup => Cow::Borrowed("Could not find round"),
                OfficalLookup => Cow::Borrowed("Could not find offical"),
                DeckLookup => Cow::Borrowed("Could not find deck"),
                RoundConfirmed => Cow::Borrowed("Round already confimed"),
                RegClosed => Cow::Borrowed("Registeration closed"),
                PlayerNotInRound => Cow::Borrowed("Player not in round"),
                NoActiveRound => Cow::Borrowed("Player has not active round"),
                IncorrectRoundStatus(status) => {
                    Cow::Owned(format!("Incorrect round status '{status}'"))
                }
                InvalidBye => Cow::Borrowed("Tried to construct an invalid bye"),
                ActiveMatches => Cow::Borrowed("Tournament currently has active matches"),
                PlayerNotCheckedIn => Cow::Borrowed("Player not checked-in"),
                IncompatiblePairingSystem => Cow::Borrowed("Incompatible pairing system"),
                IncompatibleScoringSystem => Cow::Borrowed("Incompatible scoring system"),
                InvalidDeckCount => Cow::Borrowed("Invalid deck count"),
            };
            let time = Utc::now();
            eprintln!("[FFI] {time}: {content} in tournament '{t_id}' while {context}");
        }
    }
}

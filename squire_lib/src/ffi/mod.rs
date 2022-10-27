use std::{
    alloc::{Allocator, Layout, System},
    os::raw::{c_char, c_void},
    ptr,
};

use crate::{identifiers::TournamentId, tournament::Tournament};
use dashmap::DashMap;
use once_cell::sync::OnceCell;

/// Contains the ffi C bindings for players used in SquireDesktop
#[cfg(feature = "ffi")]
pub mod ffi_player;
/// Contains the ffi C bindings for a tournament used in SquireDesktop
#[cfg(feature = "ffi")]
pub mod ffi_rounds;
/// Contains the ffi C bindings for a tournament used in SquireDesktop
#[cfg(feature = "ffi")]
pub mod ffi_tournament;

/// A map of tournament ids to tournaments
/// this is used for allocating ffi tournaments
/// all ffi tournaments are always deeply copied
/// at the lanuage barrier
pub static FFI_TOURNAMENT_REGISTRY: OnceCell<DashMap<TournamentId, Tournament>> = OnceCell::new();

/// Call this in main()
/// Inits the internal structs of squire lib for FFI.
#[no_mangle]
pub extern "C" fn init_squire_ffi() {
    FFI_TOURNAMENT_REGISTRY
        .set(DashMap::<TournamentId, Tournament>::new())
        .unwrap();
}

/// Helper function for cloning strings. Assumes that the given string is a Rust string, i.e. it
/// does not end in a NULL char. Returns NULL on error
pub fn clone_string_to_c_string(s: &str) -> *mut c_char {
    let ptr = System
        .allocate(Layout::from_size_align(s.len() + 1, 1).unwrap())
        .unwrap()
        .as_mut_ptr() as *mut c_char;

    let slice = unsafe { &mut *(ptr::slice_from_raw_parts(ptr, s.len()) as *mut [c_char]) };
    slice.iter_mut().zip(s.chars()).for_each(|(dst, c)| {
        *dst = c as i8;
    });
    slice[s.len()] = char::default() as i8;

    ptr
}

/// Deallocates a block assigned in the FFI portion,
/// use this when handling with squire strings
#[no_mangle]
pub unsafe extern "C" fn sq_free(pointer: *mut c_void, len: usize) {
    System.deallocate(
        ptr::NonNull::new(pointer as *mut u8).unwrap(),
        Layout::from_size_align(len, 1).unwrap(),
    );
}

use crate::identifiers::TournamentId;
use crate::tournament::Tournament;
use dashmap::DashMap;
use once_cell::sync::OnceCell;
use std::alloc::{Allocator, Layout, System};
use std::os::raw::c_char;
use std::ptr;

/// NULL UUIDs are returned on errors
pub const NULL_UUID_BYTES: [u8; 16] = [0; 16];

/// A map of tournament ids to tournaments
/// this is used for allocating ffi tournaments
/// all ffi tournaments are always deeply copied
/// at the lanuage barrier
pub static FFI_TOURNAMENT_REGISTRY: OnceCell<DashMap<TournamentId, Tournament>> = OnceCell::new();

#[no_mangle]
pub extern "C" fn init_squire_ffi() {
    let map: DashMap<TournamentId, Tournament> = DashMap::new();
    FFI_TOURNAMENT_REGISTRY.set(map).unwrap();
}

/// Helper function for cloning strings
pub unsafe fn clone_string_to_c_string(s: String) -> *mut c_char {
    let len: usize = s.len() + 1;
    let s_str = s.as_bytes();

    let ptr = System
        .allocate(Layout::from_size_align(len, 1).unwrap())
        .unwrap()
        .as_mut_ptr() as *mut c_char;
    let slice = &mut *(ptr::slice_from_raw_parts(ptr, len) as *mut [c_char]);
    let mut i: usize = 0;
    while i < s.len() {
        slice[i] = s_str[i] as i8;
        i += 1;
    }
    slice[i] = 0;

    return ptr;
}

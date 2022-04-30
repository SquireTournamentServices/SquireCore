#![allow(dead_code, unused_variables, unused_imports, unused_import_braces)]
#[cxx::bridge]
mod sample {
  #[namespace = "squire_core"]
  extern "Rust" {
    fn func(a: i32) -> i32;
  }
}

fn func(a: i32) -> i32 {
    a + 3
}

pub mod error;
pub mod fluid_pairings;
pub mod game;
pub mod player;
pub mod player_registry;
pub mod round;
pub mod round_registry;
pub mod scoring;
pub mod standard_scoring;
pub mod swiss_pairings;
pub mod tournament;
pub mod tournament_settings;
pub mod tournament_registry;
pub mod utils;
pub mod settings;
pub mod consts;


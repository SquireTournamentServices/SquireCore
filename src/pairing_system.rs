pub use crate::round_registry::RoundRegistry;
pub use crate::player_registry::PlayerRegistry;

pub use uuid::Uuid;

pub use std::collections::HashMap;

pub trait PairingSystem {
    fn new(players_per_match: u8) -> Self
    where
        Self: Sized;
    
    // This bool communitates if pairings should be created
    fn ready_player(&mut self) -> bool;

    fn update_settings(&mut self, settings: HashMap<String, String>) -> String;

    fn suggest_pairings(&self, players: PlayerRegistry, matches: RoundRegistry) -> Result<Vec<Vec<Uuid>>,()>;
}

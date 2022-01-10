use crate::round::Round;
use crate::player::Player;
use crate::standings::Standings;
use crate::match_registry::MatchRegistry;
use crate::pairing_system::PairingSystem;
use crate::player_registry::PlayerRegistry;
use crate::scoring_system::ScoringSystem;

use uuid::Uuid;

use std::sync::{Arc,Mutex};

pub enum TournamentPreset {
    Swiss,
    Fluid,
}

pub enum TournamentStatus {
    Registration,
    Started,
    Frozen,
    Ended,
    Cancelled,
}

pub struct Tournament {
    uuid: Uuid,
    name: String,
    format: String,
    game_size: u8,
    round_length: u64,
    deck_count: u8,
    player_reg: Arc<Mutex<PlayerRegistry>>,
    match_reg: Arc<Mutex<MatchRegistry>>,
    pairing_sys: Arc<Mutex<Box<dyn PairingSystem>>>,
    scoring_sys: Arc<Mutex<Box<dyn ScoringSystem>>>,
    reg_open: bool,
    status: TournamentStatus,
}

impl Tournament {
    pub fn from_preset(name: String, preset: TournamentPreset, format: String, game_size: u8, round_length: Duration, deck_count: u8) -> Self {
    }
    pub fn is_planned( &self ) -> bool {
        !( self.tourn_started || self.tourn_ended )
    }

    pub fn is_active( &self ) -> bool {
        self.tourn_started && !( self.tourn_ended )
    }

    pub fn is_dead( &self ) -> bool {
        self.tourn_ended
    }

    pub fn get_player( &self, identifier: String ) -> Player {
        self.player_reg.get_player( identifier )
    }

    pub fn get_round( &self, round_num: u8 ) -> Round {
        return self.match_reg.get_round( round_num )
    }

    pub fn get_standings( &self ) -> Standings {
        self.scoring_sys.get_standings()
    }

    pub fn ready_player( &self, plyr: String ) -> String {
        "You does not have a matchmaking queue.".to_string()
    }

    pub fn unready_player( &self, plyr: String ) -> String {
        "You does not have a matchmaking queue.".to_string()
    }
}

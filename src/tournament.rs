use crate::round::Round;
use crate::player::Player;
use crate::standings::Standings;
use crate::match_registry::MatchRegistry;
use crate::pairing_system::PairingSystem;
use crate::player_registry::PlayerRegistry;
use crate::scoring_system::ScoringSystem;

use uuid::Uuid;

use std::sync::{Arc,Mutex};

pub struct Tournament {
    uuid: Uuid,
    name: String,
    format: String,
    reg_open: bool,
    tourn_started: bool,
    tourn_ended: bool,
    players_per_match: u8,
    match_length: u64,
    deck_count: u8,
    player_reg: Arc<Mutex<PlayerRegistry>>,
    match_reg: Arc<Mutex<MatchRegistry>>,
    pairing_sys: Arc<Mutex<Box<dyn PairingSystem>>>,
    scoring_sys: Arc<Mutex<Box<dyn ScoringSystem>>>,
    /*
     * All of the following should be contained in an object in the TriceBot library.
    trice_bot_enabled: bool,
    spectators_allowed: bool,
    spectators_need_password: bool,
    spectators_can_chat: bool,
    spectators_can_see_hands: bool,
    only_registered: bool,
    player_deck_verification: bool,
    create_text_channel: bool,
    */
}

impl Tournament {
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

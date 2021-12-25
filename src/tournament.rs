use crate::match_registry::MatchRegistry;
use crate::pairing_system::PairingSystem;
use crate::player_registry::PlayerRegistry;
use crate::scoring_system::ScoringSystem;

use uuid::Uuid;

use std::sync::Arc;

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
    player_reg: Arc<PlayerRegistry>,
    match_reg: Arc<MatchRegistry>,
    pairing_sys: Arc<Box<dyn PairingSystem>>,
    scoring_sys: Arc<Box<dyn ScoringSystem>>,
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
}

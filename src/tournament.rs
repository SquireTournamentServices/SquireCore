use crate::fluid_pairings::FluidPairings;
use crate::pairing_system::PairingSystem;
use crate::player::Player;
use crate::player_registry::PlayerRegistry;
use crate::round::Round;
use crate::round_registry::RoundRegistry;
use crate::scoring_system::ScoringSystem;
use crate::standard_scoring::StandardScoring;
use crate::standings::Standings;
use crate::swiss_pairings::SwissPairings;
use crate::utils::{get_read_spin_lock, get_write_spin_lock};

use uuid::Uuid;

use std::sync::{Arc, RwLock};
use std::time::Duration;

pub enum TournamentPreset {
    Swiss,
    Fluid,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TournamentStatus {
    Planned,
    Started,
    Frozen,
    Ended,
    Cancelled,
}

// TODO: Consider putting the pairing and scoring systems in left-rights rather.
// Writes to those should be rare.
pub struct Tournament {
    uuid: Uuid,
    name: String,
    format: String,
    game_size: u8,
    deck_count: u8,
    player_reg: Arc<RwLock<PlayerRegistry>>,
    round_reg: Arc<RwLock<RoundRegistry>>,
    pairing_sys: Arc<RwLock<Box<dyn PairingSystem>>>,
    scoring_sys: Arc<RwLock<Box<dyn ScoringSystem>>>,
    reg_open: bool,
    status: TournamentStatus,
}

impl Tournament {
    pub fn from_preset(
        name: String,
        preset: TournamentPreset,
        format: String,
        game_size: u8,
        round_length: Duration,
        deck_count: u8,
    ) -> Self {
        let player_reg = Arc::new(RwLock::new(PlayerRegistry::new()));
        let round_reg = Arc::new(RwLock::new(RoundRegistry::new(round_length.clone())));
        let pairing_sys = Arc::new(RwLock::new(pairing_system_factory(&preset, game_size)));
        let scoring_sys = Arc::new(RwLock::new(scoring_system_factory(&preset)));
        Tournament {
            uuid: Uuid::new_v4(),
            name,
            format,
            game_size,
            deck_count,
            player_reg,
            round_reg,
            pairing_sys,
            scoring_sys,
            reg_open: true,
            status: TournamentStatus::Planned,
        }
    }
    pub fn is_planned(&self) -> bool {
        self.status == TournamentStatus::Planned
    }

    pub fn is_active(&self) -> bool {
        self.status == TournamentStatus::Started
    }

    pub fn is_dead(&self) -> bool {
        self.status == TournamentStatus::Ended || self.status == TournamentStatus::Cancelled
    }

    pub fn get_player(&self, identifier: String) -> Player {
        todo!()
        //self.player_reg.get_player( identifier )
    }

    pub fn get_round(&self, round_num: u8) -> Round {
        todo!()
        //self.match_reg.get_round( round_num )
    }

    pub fn get_standings(&self) -> Standings {
        self.scoring_sys.read().unwrap().get_standings(
            &self.player_reg.read().unwrap(),
            &self.round_reg.read().unwrap(),
        )
    }

    pub fn ready_player(&self, plyr: String) -> () {
        let sys = get_write_spin_lock(&self.pairing_sys);
    }

    pub fn unready_player(&self, plyr: String) -> String {
        todo!()
    }
}

pub fn pairing_system_factory(preset: &TournamentPreset, game_size: u8) -> Box<dyn PairingSystem> {
    match preset {
        TournamentPreset::Swiss => Box::new(SwissPairings::new(game_size)),
        TournamentPreset::Fluid => Box::new(FluidPairings::new(game_size)),
    }
}

pub fn scoring_system_factory(preset: &TournamentPreset) -> Box<dyn ScoringSystem> {
    match preset {
        TournamentPreset::Swiss => Box::new(StandardScoring::new()),
        TournamentPreset::Fluid => Box::new(StandardScoring::new()),
    }
}

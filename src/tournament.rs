use crate::fluid_pairings::FluidPairings;
use crate::pairing_system::PairingSystem;
use crate::player::Player;
use crate::player_registry::{PlayerIdentifier, PlayerRegistry};
use crate::round::Round;
use crate::round_registry::{RoundIdentifier, RoundRegistry};
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

    pub fn is_frozen(&self) -> bool {
        self.status == TournamentStatus::Frozen
    }

    pub fn is_active(&self) -> bool {
        self.status == TournamentStatus::Started
    }

    pub fn is_dead(&self) -> bool {
        self.status == TournamentStatus::Ended || self.status == TournamentStatus::Cancelled
    }

    pub fn update_reg(&mut self, reg_status: bool) -> () {
        self.reg_open = reg_status;
    }

    pub fn start(&mut self) -> Result<(), ()> {
        if !self.is_planned() {
            Err(())
        } else {
            self.reg_open = false;
            self.status = TournamentStatus::Started;
            Ok(())
        }
    }

    pub fn freeze(&mut self) -> Result<(), ()> {
        if !self.is_active() {
            Err(())
        } else {
            self.reg_open = false;
            self.status = TournamentStatus::Frozen;
            Ok(())
        }
    }

    pub fn thaw(&mut self) -> Result<(), ()> {
        if !self.is_frozen() {
            Err(())
        } else {
            self.status = TournamentStatus::Started;
            Ok(())
        }
    }

    pub fn end(&mut self) -> Result<(), ()> {
        if !self.is_active() {
            Err(())
        } else {
            self.reg_open = false;
            self.status = TournamentStatus::Ended;
            Ok(())
        }
    }

    pub fn cancel(&mut self) -> Result<(), ()> {
        if !self.is_active() {
            Err(())
        } else {
            self.reg_open = false;
            self.status = TournamentStatus::Cancelled;
            Ok(())
        }
    }

    pub fn admin_drop_player(&self, ident: PlayerIdentifier) -> Result<(), ()> {
        let player_lock = get_write_spin_lock(&self.player_reg);
        player_lock.remove_player(ident)
    }
    
    pub fn get_player(&self, identifier: String) -> Player {
        todo!()
    }

    pub fn get_round(&self, round_num: u8) -> Round {
        todo!()
    }

    pub fn get_standings(&self) -> Standings {
        let sys = get_read_spin_lock(&self.scoring_sys);
        sys.get_standings(
            &self.player_reg.read().unwrap(),
            &self.round_reg.read().unwrap(),
        )
    }

    pub fn ready_player(&self, plyr: String) -> () {
        todo!()
    }

    pub fn unready_player(&self, plyr: String) -> String {
        todo!()
    }

    pub fn set_deck_count(&mut self, deck_count: u8) -> () {
        self.deck_count = deck_count;
    }

    pub fn set_game_size(&mut self, game_size: u8) -> () {
        self.game_size = game_size;
    }

    pub fn set_round_length(&self, length: Duration) -> () {
        let mut sys = get_write_spin_lock(&self.round_reg);
        sys.set_round_length(length);
    }
    
    pub fn give_bye(&self, ident: PlayerIdentifier) -> Result<(),()> {
        let player_lock = get_read_spin_lock(&self.player_reg);
        if player_lock.verify_identifier(&ident) {
            let id = player_lock.get_player_id(ident).unwrap();
            let mut round_lock = get_write_spin_lock(&self.round_reg);
            let round = round_lock.create_round();
            round.add_player(id);
            round.record_bye();
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn create_round(&self, idents: Vec<PlayerIdentifier>) -> Result<(), ()> {
        let player_lock = get_read_spin_lock(&self.player_reg);
        if idents.len() == self.game_size as usize && idents.iter().all(|p| !player_lock.verify_identifier(p)) {
            // Saftey check, we already checked that all the identifiers correspond to a player
            let ids: Vec<Uuid> = idents
                .into_iter()
                .map(|p| player_lock.get_player_id(p).unwrap())
                .collect();
            let mut round_lock = get_write_spin_lock(&self.round_reg);
            let round = round_lock.create_round();
            for id in ids {
                round.add_player(id);
            }
            Ok(())
        } else {
            Err(())
        }
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

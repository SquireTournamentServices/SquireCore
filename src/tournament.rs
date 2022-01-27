use crate::error::TournamentError;
use crate::fluid_pairings::FluidPairings;
use crate::pairing_system::PairingSystem;
use crate::player::Player;
use crate::player_registry::{PlayerIdentifier, PlayerRegistry};
use crate::round::{parse_to_outcome, Round, RoundStatus};
use crate::round_registry::{RoundIdentifier, RoundRegistry};
use crate::scoring_system::ScoringSystem;
use crate::standard_scoring::StandardScoring;
use crate::standings::Standings;
use crate::swiss_pairings::SwissPairings;
use crate::utils::{get_read_spin_lock, get_write_spin_lock};

use mtgjson::model::deck::Deck;
use uuid::Uuid;

use std::collections::HashMap;
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
        let round_reg = Arc::new(RwLock::new(RoundRegistry::new(round_length)));
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

    pub fn update_reg(&mut self, reg_status: bool) {
        self.reg_open = reg_status;
    }

    pub fn start(&mut self) -> Result<(), TournamentError> {
        if !self.is_planned() {
            Err(TournamentError::IncorrectStatus)
        } else {
            self.reg_open = false;
            self.status = TournamentStatus::Started;
            Ok(())
        }
    }

    pub fn freeze(&mut self) -> Result<(), TournamentError> {
        if !self.is_active() {
            Err(TournamentError::IncorrectStatus)
        } else {
            self.reg_open = false;
            self.status = TournamentStatus::Frozen;
            Ok(())
        }
    }

    pub fn thaw(&mut self) -> Result<(), TournamentError> {
        if !self.is_frozen() {
            Err(TournamentError::IncorrectStatus)
        } else {
            self.status = TournamentStatus::Started;
            Ok(())
        }
    }

    pub fn end(&mut self) -> Result<(), TournamentError> {
        if !self.is_active() {
            Err(TournamentError::IncorrectStatus)
        } else {
            self.reg_open = false;
            self.status = TournamentStatus::Ended;
            Ok(())
        }
    }

    pub fn cancel(&mut self) -> Result<(), TournamentError> {
        if !self.is_active() {
            Err(TournamentError::IncorrectStatus)
        } else {
            self.reg_open = false;
            self.status = TournamentStatus::Cancelled;
            Ok(())
        }
    }

    pub fn register_player(&self, name: String) -> Result<(), TournamentError> {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus);
        }
        let mut player_lock = get_write_spin_lock(&self.player_reg);
        player_lock.add_player(name)
    }

    pub fn record_outcome(
        &self,
        ident: RoundIdentifier,
        input: String,
    ) -> Result<(), TournamentError> {
        let outcome = parse_to_outcome(input)?;
        let mut round_lock = get_write_spin_lock(&self.round_reg);
        let round = round_lock.get_mut_round(ident)?;
        round.record_outcome(outcome)?;
        Ok(())
    }

    pub fn confirm_round(&self, ident: PlayerIdentifier) -> Result<RoundStatus, TournamentError> {
        let player_lock = get_read_spin_lock(&self.player_reg);
        let id = player_lock.get_player_id(ident)?;
        drop(player_lock);
        let mut round_lock = get_write_spin_lock(&self.round_reg);
        let round = round_lock.get_player_active_round(id)?;
        round.confirm_round(id)
    }

    pub fn admin_drop_player(&self, ident: PlayerIdentifier) -> Result<(), TournamentError> {
        let mut player_lock = get_write_spin_lock(&self.player_reg);
        player_lock.remove_player(ident)
    }

    pub fn player_add_deck(
        &self,
        ident: PlayerIdentifier,
        name: String,
        deck: Deck,
    ) -> Result<(), TournamentError> {
        let mut player_lock = get_write_spin_lock(&self.player_reg);
        let plyr = player_lock.get_mut_player(ident)?;
        plyr.add_deck(name, deck);
        Ok(())
    }

    pub fn get_player_decks(
        &self,
        ident: PlayerIdentifier,
    ) -> Result<HashMap<String, Deck>, TournamentError> {
        let player_lock = get_read_spin_lock(&self.player_reg);
        let plyr = player_lock.get_player(ident)?;
        Ok(plyr.get_decks())
    }

    pub fn remove_player_deck(
        &self,
        ident: PlayerIdentifier,
        name: String,
    ) -> Result<(), TournamentError> {
        let mut player_lock = get_write_spin_lock(&self.player_reg);
        let plyr = player_lock.get_mut_player(ident)?;
        plyr.remove_deck(name)
    }

    pub fn get_player_deck(
        &self,
        ident: PlayerIdentifier,
        name: String,
    ) -> Result<Deck, TournamentError> {
        let player_lock = get_read_spin_lock(&self.player_reg);
        let plyr = player_lock.get_player(ident)?;
        match plyr.get_deck(name) {
            None => Err(TournamentError::DeckLookup),
            Some(d) => Ok(d),
        }
    }

    pub fn player_set_game_name(
        &self,
        ident: PlayerIdentifier,
        name: String,
    ) -> Result<(), TournamentError> {
        let mut player_lock = get_write_spin_lock(&self.player_reg);
        let plyr = player_lock.get_mut_player(ident)?;
        plyr.set_game_name(name);
        Ok(())
    }

    pub fn get_standings(&self) -> Standings {
        let sys = get_read_spin_lock(&self.scoring_sys);
        sys.get_standings(
            &self.player_reg.read().unwrap(),
            &self.round_reg.read().unwrap(),
        )
    }

    pub fn ready_player(&self, ident: PlayerIdentifier) -> Result<(), TournamentError> {
        let mut pairing_lock = get_write_spin_lock(&self.pairing_sys);
        let player_lock = get_read_spin_lock(&self.player_reg);
        let plyr = player_lock.get_player(ident)?;
        let mut should_pair = false;
        if plyr.can_play() {
            should_pair = pairing_lock.ready_player(plyr.uuid);
        }
        if should_pair {
            let mut round_lock = get_write_spin_lock(&self.round_reg);
            if let Some(pairings) =
                pairing_lock.suggest_pairings(self.game_size, &player_lock, &round_lock)
            {
                for p in pairings {
                    let round = round_lock.create_round();
                    for plyr in p {
                        round.add_player(plyr);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn unready_player(&self, plyr: String) -> String {
        todo!()
    }

    pub fn set_deck_count(&mut self, deck_count: u8) {
        self.deck_count = deck_count;
    }

    pub fn set_game_size(&mut self, game_size: u8) {
        self.game_size = game_size;
    }

    pub fn set_round_length(&self, length: Duration) {
        let mut sys = get_write_spin_lock(&self.round_reg);
        sys.set_round_length(length);
    }

    pub fn give_bye(&self, ident: PlayerIdentifier) -> Result<(), TournamentError> {
        let player_lock = get_read_spin_lock(&self.player_reg);
        let id = player_lock.get_player_id(ident)?;
        let mut round_lock = get_write_spin_lock(&self.round_reg);
        let round = round_lock.create_round();
        round.add_player(id);
        // Saftey check: This should never return an Err as we just created the round and gave it a
        // single player
        let _ = round.record_bye();
        Ok(())
    }

    pub fn create_round(&self, idents: Vec<PlayerIdentifier>) -> Result<(), TournamentError> {
        let player_lock = get_read_spin_lock(&self.player_reg);
        if idents.len() == self.game_size as usize
            && idents.iter().all(|p| !player_lock.verify_identifier(p))
        {
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
            Err(TournamentError::PlayerLookup)
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

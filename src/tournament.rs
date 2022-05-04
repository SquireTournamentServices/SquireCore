use crate::{
    error::TournamentError,
    fluid_pairings::FluidPairings,
    player::{Player, PlayerId},
    player_registry::{PlayerIdentifier, PlayerRegistry},
    round::{parse_to_outcome, Round, RoundId, RoundStatus},
    round_registry::{RoundIdentifier, RoundRegistry},
    scoring::{Score, Standings},
    standard_scoring::{StandardScore, StandardScoring},
    swiss_pairings::SwissPairings,
    tournament_settings::TournamentSettings,
};

use mtgjson::model::deck::Deck;
use uuid::Uuid;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::Duration;

#[repr(C)]
pub enum TournamentPreset {
    Swiss,
    Fluid,
}

#[repr(C)]
pub enum ScoringSystem {
    Standard(StandardScoring),
}

#[repr(C)]
pub enum PairingSystem {
    Swiss(SwissPairings),
    Fluid(FluidPairings),
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TournamentStatus {
    Planned,
    Started,
    Frozen,
    Ended,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(C)]
pub struct TournamentId(Uuid);

#[repr(C)]
pub struct Tournament {
    id: TournamentId,
    pub name: String,
    format: String,
    game_size: u8,
    deck_count: u8,
    player_reg: PlayerRegistry,
    round_reg: RoundRegistry,
    pairing_sys: PairingSystem,
    scoring_sys: ScoringSystem,
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
        Tournament {
            id: TournamentId(Uuid::new_v4()),
            name,
            format,
            game_size,
            deck_count,
            player_reg: PlayerRegistry::new(),
            round_reg: RoundRegistry::new(round_length),
            pairing_sys: pairing_system_factory(&preset, game_size),
            scoring_sys: scoring_system_factory(&preset),
            reg_open: true,
            status: TournamentStatus::Planned,
        }
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

    pub fn get_player(&self, ident: &PlayerIdentifier) -> Result<Player, TournamentError> {
        match self.player_reg.get_player(ident) {
            Some(plyr) => Ok(plyr.clone()),
            None => Err(TournamentError::PlayerLookup),
        }
    }

    pub fn get_round(&self, ident: &RoundIdentifier) -> Result<Round, TournamentError> {
        match self.round_reg.get_round(ident) {
            Some(rnd) => Ok(rnd.clone()),
            None => Err(TournamentError::RoundLookup),
        }
    }

    pub fn register_player(&mut self, name: String) -> Result<PlayerId, TournamentError> {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus);
        }
        if !self.reg_open {
            return Err(TournamentError::RegClosed);
        }
        self.player_reg.add_player(name)
    }

    pub fn record_outcome(
        &mut self,
        ident: RoundIdentifier,
        input: String,
    ) -> Result<(), TournamentError> {
        let outcome = parse_to_outcome(input)?;
        let round = self
            .round_reg
            .get_mut_round(ident)
            .ok_or(TournamentError::RoundLookup)?;
        round.record_outcome(outcome)?;
        Ok(())
    }

    pub fn confirm_round(
        &mut self,
        ident: &PlayerIdentifier,
    ) -> Result<RoundStatus, TournamentError> {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus);
        }
        let id = self
            .player_reg
            .get_player_id(ident)
            .ok_or(TournamentError::PlayerLookup)?;
        let round = self.round_reg.get_player_active_round(id)?;
        round.confirm_round(id)
    }

    pub fn drop_player(&mut self, ident: &PlayerIdentifier) -> Result<(), TournamentError> {
        self.player_reg
            .remove_player(ident)
            .ok_or(TournamentError::PlayerLookup)
    }

    pub fn admin_drop_player(&mut self, ident: &PlayerIdentifier) -> Result<(), TournamentError> {
        self.player_reg
            .remove_player(ident)
            .ok_or(TournamentError::PlayerLookup)
    }

    pub fn player_add_deck(
        &mut self,
        ident: &PlayerIdentifier,
        name: String,
        deck: Deck,
    ) -> Result<(), TournamentError> {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus);
        }
        if !self.reg_open {
            return Err(TournamentError::RegClosed);
        }
        let plyr = self
            .player_reg
            .get_mut_player(ident)
            .ok_or(TournamentError::PlayerLookup)?;
        plyr.add_deck(name, deck);
        Ok(())
    }

    pub fn get_player_decks(
        &self,
        ident: &PlayerIdentifier,
    ) -> Result<HashMap<String, Deck>, TournamentError> {
        let plyr = self
            .player_reg
            .get_player(&ident)
            .ok_or(TournamentError::PlayerLookup)?;
        Ok(plyr.get_decks())
    }

    pub fn remove_player_deck(
        &mut self,
        ident: &PlayerIdentifier,
        name: String,
    ) -> Result<(), TournamentError> {
        let plyr = self
            .player_reg
            .get_mut_player(ident)
            .ok_or(TournamentError::PlayerLookup)?;
        plyr.remove_deck(name)
    }

    pub fn get_player_deck(
        &mut self,
        ident: &PlayerIdentifier,
        name: String,
    ) -> Result<Deck, TournamentError> {
        let plyr = self
            .player_reg
            .get_player(ident)
            .ok_or(TournamentError::PlayerLookup)?;
        match plyr.get_deck(name) {
            None => Err(TournamentError::DeckLookup),
            Some(d) => Ok(d),
        }
    }

    pub fn get_player_round(&self, ident: &PlayerIdentifier) -> Result<RoundId, TournamentError> {
        let p_id = self
            .player_reg
            .get_player_id(ident)
            .ok_or(TournamentError::PlayerLookup)?;
        let rounds: Vec<RoundId> = self
            .round_reg
            .rounds
            .iter()
            .filter(|(_, r)| r.players.contains(&p_id))
            .map(|(_, r)| r.id.clone())
            .collect();
        if rounds.len() == 1 {
            Ok(rounds[0])
        } else {
            Err(TournamentError::RoundLookup)
        }
    }

    pub fn player_set_game_name(
        &mut self,
        ident: &PlayerIdentifier,
        name: String,
    ) -> Result<(), TournamentError> {
        let plyr = self
            .player_reg
            .get_mut_player(ident)
            .ok_or(TournamentError::PlayerLookup)?;
        plyr.set_game_name(name);
        Ok(())
    }

    pub fn get_standings(&self) -> Standings<StandardScore> {
        self.scoring_sys
            .get_standings(&self.player_reg, &self.round_reg)
    }

    pub fn ready_player(&mut self, ident: &PlayerIdentifier) -> Result<(), TournamentError> {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus);
        }
        let plyr = self
            .player_reg
            .get_player(ident)
            .ok_or(TournamentError::PlayerLookup)?;
        let mut should_pair = false;
        if plyr.can_play() {
            should_pair = self.pairing_sys.ready_player(plyr.id);
        }
        if should_pair {
            if let Some(pairings) =
                self.pairing_sys
                    .suggest_pairings(self.game_size, &self.player_reg, &self.round_reg)
            {
                for p in pairings {
                    let round = self.round_reg.create_round();
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

    pub fn set_round_length(&mut self, length: Duration) {
        self.round_reg.set_round_length(length);
    }

    pub fn give_bye(&mut self, ident: &PlayerIdentifier) -> Result<(), TournamentError> {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus);
        }
        let id = self
            .player_reg
            .get_player_id(ident)
            .ok_or(TournamentError::PlayerLookup)?;
        let round = self.round_reg.create_round();
        round.add_player(id);
        // Saftey check: This should never return an Err as we just created the round and gave it a
        // single player
        let _ = round.record_bye();
        Ok(())
    }

    pub fn create_round(&mut self, idents: Vec<PlayerIdentifier>) -> Result<(), TournamentError> {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus);
        }
        if idents.len() == self.game_size as usize
            && idents.iter().all(|p| !self.player_reg.verify_identifier(p))
        {
            // Saftey check, we already checked that all the identifiers correspond to a player
            let ids: Vec<PlayerId> = idents
                .into_iter()
                .map(|p| self.player_reg.get_player_id(&p).unwrap())
                .collect();
            let round = self.round_reg.create_round();
            for id in ids {
                round.add_player(id);
            }
            Ok(())
        } else {
            Err(TournamentError::PlayerLookup)
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
}

#[cfg(feature = "ffi")]
impl Tournament {
    pub extern "C" fn from_preset_c(
        name: String,
        preset: TournamentPreset,
        format: String,
        game_size: u8,
        round_length: Duration,
        deck_count: u8,
    ) -> Self {
        Tournament {
            id: TournamentId(Uuid::new_v4()),
            name,
            format,
            game_size,
            deck_count,
            player_reg: PlayerRegistry::new(),
            round_reg: RoundRegistry::new(round_length),
            pairing_sys: pairing_system_factory(&preset, game_size),
            scoring_sys: scoring_system_factory(&preset),
            reg_open: true,
            status: TournamentStatus::Planned,
        }
    }

    pub extern "C" fn update_reg_c(&mut self, reg_status: bool) {
        self.reg_open = reg_status;
    }

    pub extern "C" fn start_c(&mut self) -> Result<(), TournamentError> {
        if !self.is_planned() {
            Err(TournamentError::IncorrectStatus)
        } else {
            self.reg_open = false;
            self.status = TournamentStatus::Started;
            Ok(())
        }
    }
}

pub fn pairing_system_factory(preset: &TournamentPreset, game_size: u8) -> PairingSystem {
    match preset {
        TournamentPreset::Swiss => PairingSystem::Swiss(SwissPairings::new(game_size)),
        TournamentPreset::Fluid => PairingSystem::Fluid(FluidPairings::new(game_size)),
    }
}

pub fn scoring_system_factory(preset: &TournamentPreset) -> ScoringSystem {
    match preset {
        TournamentPreset::Swiss => ScoringSystem::Standard(StandardScoring::new()),
        TournamentPreset::Fluid => ScoringSystem::Standard(StandardScoring::new()),
    }
}

impl Hash for Tournament {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        let _ = &self.id.hash(state);
    }
}
impl PairingSystem {
    pub fn ready_player(&mut self, id: PlayerId) -> bool {
        match self {
            Self::Swiss(sys) => sys.ready_player(id),
            Self::Fluid(sys) => sys.ready_player(id),
        }
    }

    pub fn suggest_pairings(
        &mut self,
        size: u8,
        plyr_reg: &PlayerRegistry,
        rnd_reg: &RoundRegistry,
    ) -> Option<Vec<Vec<PlayerId>>> {
        match self {
            Self::Swiss(sys) => sys.suggest_pairings(size, plyr_reg, rnd_reg),
            Self::Fluid(sys) => sys.suggest_pairings(size, plyr_reg, rnd_reg),
        }
    }
}

impl ScoringSystem {
    pub fn get_standings(
        &self,
        player_reg: &PlayerRegistry,
        round_reg: &RoundRegistry,
    ) -> Standings<StandardScore> {
        match self {
            ScoringSystem::Standard(s) => s.get_standings(player_reg, round_reg),
        }
    }
}

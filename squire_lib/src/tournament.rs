use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    hash::{Hash, Hasher},
    str::Utf8Error,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use mtgjson::model::deck::Deck;

use crate::{
    error::TournamentError,
    fluid_pairings::FluidPairings,
    operations::{OpData, OpResult, TournOp},
    pairings::Pairings,
    player::{Player, PlayerId, PlayerStatus},
    player_registry::{PlayerIdentifier, PlayerRegistry},
    round::{Round, RoundId, RoundResult, RoundStatus},
    round_registry::{RoundIdentifier, RoundRegistry},
    scoring::{Score, Standings},
    settings::{
        self, FluidPairingsSetting, PairingSetting, ScoringSetting, StandardScoringSetting,
        SwissPairingsSetting, TournamentSetting,
    },
    standard_scoring::{StandardScore, StandardScoring},
    swiss_pairings::SwissPairings,
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum TournamentPreset {
    Swiss,
    Fluid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ScoringSystem {
    Standard(StandardScoring),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PairingSystem {
    Swiss(SwissPairings),
    Fluid(FluidPairings),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum TournamentStatus {
    Planned,
    Started,
    Frozen,
    Ended,
    Cancelled,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, Copy)]
#[repr(C)]
pub struct TournamentId(pub Uuid);

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub enum TournamentIdentifier {
    Id(TournamentId),
    Name(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tournament {
    pub id: TournamentId,
    pub name: String,
    pub use_table_number: bool,
    pub format: String,
    pub game_size: u8,
    pub min_deck_count: u8,
    pub max_deck_count: u8,
    pub player_reg: PlayerRegistry,
    pub round_reg: RoundRegistry,
    pub pairing_sys: PairingSystem,
    pub scoring_sys: ScoringSystem,
    pub reg_open: bool,
    pub require_check_in: bool,
    pub require_deck_reg: bool,
    pub status: TournamentStatus,
}

impl Tournament {
    pub fn from_preset(name: String, preset: TournamentPreset, format: String) -> Self {
        Tournament {
            id: TournamentId(Uuid::new_v4()),
            name,
            use_table_number: true,
            format,
            game_size: 2,
            min_deck_count: 1,
            max_deck_count: 2,
            player_reg: PlayerRegistry::new(),
            round_reg: RoundRegistry::new(0, Duration::from_secs(3000)),
            pairing_sys: pairing_system_factory(&preset, 2),
            scoring_sys: scoring_system_factory(&preset),
            reg_open: true,
            require_check_in: false,
            require_deck_reg: false,
            status: TournamentStatus::Planned,
        }
    }

    pub fn apply_op(&mut self, op: TournOp) -> OpResult {
        use TournOp::*;
        match op {
            UpdateReg(b) => self.update_reg(b),
            Start() => self.start(),
            Freeze() => self.freeze(),
            Thaw() => self.thaw(),
            End() => self.end(),
            Cancel() => self.cancel(),
            CheckIn(p_ident) => self.check_in(&p_ident),
            RegisterPlayer(name) => self.register_player(name),
            RecordResult(r_ident, result) => self.record_result(&r_ident, result),
            ConfirmResult(p_ident) => self.confirm_round(&p_ident),
            DropPlayer(p_ident) => self.drop_player(&p_ident),
            AdminDropPlayer(p_ident) => self.admin_drop_player(&p_ident),
            AddDeck(p_ident, name, deck) => self.player_add_deck(&p_ident, name, deck),
            RemoveDeck(p_ident, name) => self.remove_player_deck(&p_ident, name),
            RemoveRound(r_ident) => self.remove_round(&r_ident),
            SetGamerTag(p_ident, tag) => self.player_set_game_name(&p_ident, tag),
            ReadyPlayer(p_ident) => self.ready_player(&p_ident),
            UnReadyPlayer(p_ident) => self.unready_player(&p_ident),
            UpdateTournSetting(setting) => self.update_setting(setting),
            GiveBye(p_ident) => self.give_bye(&p_ident),
            CreateRound(p_idents) => self.create_round(p_idents),
            PairRound() => self.pair(),
            TimeExtension(rnd, ext) => self.give_time_extension(&rnd, ext),
            Cut(n) => self.cut_to_top(n),
            PruneDecks() => self.prune_decks(),
            PrunePlayers() => self.prune_players(),
            ImportPlayer(plyr) => self.import_player(plyr),
            ImportRound(rnd) => self.import_round(rnd),
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

    pub fn get_player_rounds(
        &self,
        ident: &PlayerIdentifier,
    ) -> Result<Vec<Round>, TournamentError> {
        let id = self
            .player_reg
            .get_player_id(ident)
            .ok_or(TournamentError::PlayerLookup)?;
        Ok(self
            .round_reg
            .rounds
            .iter()
            .filter_map(|(_, r)| r.players.get(&id).map(|_| r.clone()))
            .collect())
    }

    pub fn get_player_deck(
        &self,
        ident: &PlayerIdentifier,
        name: &String,
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

    pub fn get_standings(&self) -> Standings<StandardScore> {
        self.scoring_sys
            .get_standings(&self.player_reg, &self.round_reg)
    }

    pub(crate) fn prune_decks(&mut self) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if self.require_deck_reg {
            for (_, p) in self.player_reg.players.iter_mut() {
                while p.decks.len() > self.max_deck_count as usize {
                    let name = p.deck_ordering[0].clone();
                    let _ = p.remove_deck(name);
                }
            }
        }
        Ok(OpData::Nothing)
    }

    pub(crate) fn import_player(&mut self, plyr: Player) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        self.player_reg.import_player(plyr)?;
        Ok(OpData::Nothing)
    }

    pub(crate) fn import_round(&mut self, rnd: Round) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        self.round_reg.import_round(rnd)?;
        Ok(OpData::Nothing)
    }

    pub(crate) fn prune_players(&mut self) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if self.require_deck_reg {
            for (_, p) in self.player_reg.players.iter_mut() {
                if p.decks.len() < self.min_deck_count as usize {
                    p.update_status(PlayerStatus::Dropped);
                }
            }
        }
        if self.require_check_in {
            for (id, p) in self.player_reg.players.iter_mut() {
                if !self.player_reg.check_ins.contains(id) {
                    p.update_status(PlayerStatus::Dropped);
                }
            }
        }
        Ok(OpData::Nothing)
    }

    pub(crate) fn give_time_extension(&mut self, rnd: &RoundIdentifier, ext: Duration) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let round = self
            .round_reg
            .get_mut_round(rnd)
            .ok_or(TournamentError::RoundLookup)?;
        round.extension += ext;
        Ok(OpData::Nothing)
    }

    pub(crate) fn check_in(&mut self, plyr: &PlayerIdentifier) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let id = self
            .player_reg
            .get_player_id(plyr)
            .ok_or(TournamentError::PlayerLookup)?;
        if self.is_planned() {
            self.player_reg.check_in(id);
            Ok(OpData::Nothing)
        } else {
            Err(TournamentError::IncorrectStatus(self.status))
        }
    }

    pub(crate) fn pair(&mut self) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let standings = self
            .scoring_sys
            .get_standings(&self.player_reg, &self.round_reg);
        if let Some(pairings) = self
            .pairing_sys
            .pair(&self.player_reg, &self.round_reg, standings)
        {
            let mut rounds = Vec::with_capacity(pairings.paired.len());
            for pair in pairings.paired {
                let r_id = self.round_reg.create_round();
                for plyr in pair {
                    let _ = self.round_reg.add_player_to_round(&r_id, plyr);
                }
                rounds.push(r_id);
            }
            if let PairingSystem::Swiss(_) = &self.pairing_sys {
                for plyr in pairings.rejected {
                    let r_id = self.round_reg.create_round();
                    let rnd = self.round_reg.get_mut_round(&r_id).unwrap();
                    rnd.add_player(plyr);
                    let _ = rnd.record_bye();
                    rounds.push(r_id);
                }
            }
            Ok(OpData::Pair(rounds))
        } else {
            Ok(OpData::Nothing)
        }
    }

    pub(crate) fn remove_round(&mut self, ident: &RoundIdentifier) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        self.round_reg.kill_round(ident)?;
        Ok(OpData::Nothing)
    }

    pub(crate) fn update_setting(&mut self, setting: TournamentSetting) -> OpResult {
        use TournamentSetting::*;
        if self.is_dead() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        match setting {
            Format(f) => {
                self.format = f;
            }
            StartingTableNumber(n) => {
                self.round_reg.starting_table = n;
            }
            UseTableNumbers(b) => {
                self.use_table_number = b;
            }
            MinDeckCount(c) => {
                self.min_deck_count = c;
            }
            MaxDeckCount(c) => {
                self.max_deck_count = c;
            }
            RequireCheckIn(b) => {
                self.require_check_in = b;
            }
            RequireDeckReg(b) => {
                self.require_deck_reg = b;
            }
            PairingSetting(setting) => match setting {
                settings::PairingSetting::Swiss(s) => {
                    if let PairingSystem::Swiss(sys) = &mut self.pairing_sys {
                        sys.update_setting(s);
                    } else {
                        return Err(TournamentError::IncompatiblePairingSystem);
                    }
                }
                settings::PairingSetting::Fluid(s) => {
                    if let PairingSystem::Fluid(sys) = &mut self.pairing_sys {
                        sys.update_setting(s);
                    } else {
                        return Err(TournamentError::IncompatiblePairingSystem);
                    }
                }
            },
            ScoringSetting(setting) => match setting {
                settings::ScoringSetting::Standard(s) => {
                    if let ScoringSystem::Standard(sys) = &mut self.scoring_sys {
                        sys.update_setting(s);
                    } else {
                        return Err(TournamentError::IncompatibleScoringSystem);
                    }
                }
            },
        }
        Ok(OpData::Nothing)
    }

    pub(crate) fn update_reg(&mut self, reg_status: bool) -> OpResult {
        if self.is_dead() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        self.reg_open = reg_status;
        Ok(OpData::Nothing)
    }

    pub(crate) fn start(&mut self) -> OpResult {
        if !self.is_planned() {
            Err(TournamentError::IncorrectStatus(self.status))
        } else {
            self.reg_open = false;
            self.status = TournamentStatus::Started;
            Ok(OpData::Nothing)
        }
    }

    pub(crate) fn freeze(&mut self) -> OpResult {
        if !self.is_active() {
            Err(TournamentError::IncorrectStatus(self.status))
        } else {
            self.reg_open = false;
            self.status = TournamentStatus::Frozen;
            Ok(OpData::Nothing)
        }
    }

    pub(crate) fn thaw(&mut self) -> OpResult {
        if !self.is_frozen() {
            Err(TournamentError::IncorrectStatus(self.status))
        } else {
            self.status = TournamentStatus::Started;
            Ok(OpData::Nothing)
        }
    }

    pub(crate) fn end(&mut self) -> OpResult {
        if !self.is_active() {
            Err(TournamentError::IncorrectStatus(self.status))
        } else {
            self.reg_open = false;
            self.status = TournamentStatus::Ended;
            Ok(OpData::Nothing)
        }
    }

    pub(crate) fn cancel(&mut self) -> OpResult {
        if !self.is_active() {
            Err(TournamentError::IncorrectStatus(self.status))
        } else {
            self.reg_open = false;
            self.status = TournamentStatus::Cancelled;
            Ok(OpData::Nothing)
        }
    }

    pub(crate) fn register_player(&mut self, name: String) -> OpResult {
        if !(self.is_active() || self.is_planned()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if !self.reg_open {
            return Err(TournamentError::RegClosed);
        }
        let id = self.player_reg.add_player(name)?;
        Ok(OpData::RegisterPlayer(PlayerIdentifier::Id(id)))
    }

    pub(crate) fn record_result(
        &mut self,
        ident: &RoundIdentifier,
        result: RoundResult,
    ) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let round = self
            .round_reg
            .get_mut_round(ident)
            .ok_or(TournamentError::RoundLookup)?;
        round.record_result(result)?;
        Ok(OpData::Nothing)
    }

    pub(crate) fn confirm_round(&mut self, ident: &PlayerIdentifier) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let id = self
            .player_reg
            .get_player_id(ident)
            .ok_or(TournamentError::PlayerLookup)?;
        let round = self.round_reg.get_player_active_round(&id)?;
        let status = round.confirm_round(id)?;
        Ok(OpData::ConfirmResult(status))
    }

    pub(crate) fn drop_player(&mut self, ident: &PlayerIdentifier) -> OpResult {
        if self.is_dead() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        self.player_reg
            .remove_player(ident)
            .ok_or(TournamentError::PlayerLookup)?;
        Ok(OpData::Nothing)
    }

    pub(crate) fn admin_drop_player(&mut self, ident: &PlayerIdentifier) -> OpResult {
        if self.is_dead() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        self.player_reg
            .remove_player(ident)
            .ok_or(TournamentError::PlayerLookup)?;
        Ok(OpData::Nothing)
    }

    pub(crate) fn player_add_deck(
        &mut self,
        ident: &PlayerIdentifier,
        name: String,
        deck: Deck,
    ) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if !self.reg_open {
            return Err(TournamentError::RegClosed);
        }
        let plyr = self
            .player_reg
            .get_mut_player(ident)
            .ok_or(TournamentError::PlayerLookup)?;
        plyr.add_deck(name, deck);
        Ok(OpData::Nothing)
    }

    pub(crate) fn get_player_decks(
        &self,
        ident: &PlayerIdentifier,
    ) -> Result<HashMap<String, Deck>, TournamentError> {
        let plyr = self
            .player_reg
            .get_player(ident)
            .ok_or(TournamentError::PlayerLookup)?;
        Ok(plyr.get_decks())
    }

    pub(crate) fn remove_player_deck(
        &mut self,
        ident: &PlayerIdentifier,
        name: String,
    ) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let plyr = self
            .player_reg
            .get_mut_player(ident)
            .ok_or(TournamentError::PlayerLookup)?;
        plyr.remove_deck(name)?;
        Ok(OpData::Nothing)
    }

    pub(crate) fn player_set_game_name(
        &mut self,
        ident: &PlayerIdentifier,
        name: String,
    ) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let plyr = self
            .player_reg
            .get_mut_player(ident)
            .ok_or(TournamentError::PlayerLookup)?;
        plyr.set_game_name(name);
        Ok(OpData::Nothing)
    }

    pub(crate) fn ready_player(&mut self, ident: &PlayerIdentifier) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let plyr = self
            .player_reg
            .get_player(ident)
            .ok_or(TournamentError::PlayerLookup)?;
        let mut should_pair = false;
        if plyr.can_play() {
            self.pairing_sys.ready_player(plyr.id.clone());
            should_pair = match &self.pairing_sys {
                PairingSystem::Fluid(sys) => sys.ready_to_pair(),
                PairingSystem::Swiss(sys) => false,
            };
        }
        if should_pair {
            let standings = self.get_standings();
            if let Some(pairings) =
                self.pairing_sys
                    .pair(&self.player_reg, &self.round_reg, standings)
            {
                let mut rounds = Vec::with_capacity(pairings.paired.len());
                for p in pairings.paired {
                    let r_id = self.round_reg.create_round();
                    for plyr in p {
                        let _ = self.round_reg.add_player_to_round(&r_id, plyr);
                    }
                    rounds.push(r_id);
                }
                return Ok(OpData::Pair(rounds));
            }
        }
        Ok(OpData::Nothing)
    }

    pub(crate) fn unready_player(&mut self, plyr: &PlayerIdentifier) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let plyr = self
            .player_reg
            .get_player_id(plyr)
            .ok_or(TournamentError::PlayerLookup)?;
        match &mut self.pairing_sys {
            PairingSystem::Swiss(sys) => sys.unready_player(plyr),
            PairingSystem::Fluid(sys) => sys.unready_player(plyr),
        };
        Ok(OpData::Nothing)
    }

    pub(crate) fn give_bye(&mut self, ident: &PlayerIdentifier) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let id = self
            .player_reg
            .get_player_id(ident)
            .ok_or(TournamentError::PlayerLookup)?;
        let r_id = self.round_reg.create_round();
        let _ = self.round_reg.add_player_to_round(&r_id, id);
        // Saftey check: This should never return an Err as we just created the round and gave it a
        // single player
        let round = self.round_reg.get_mut_round(&r_id).unwrap();
        round.record_bye()?;
        Ok(OpData::GiveBye(r_id))
    }

    pub(crate) fn create_round(&mut self, idents: Vec<PlayerIdentifier>) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if idents.len() == self.game_size as usize
            && idents.iter().all(|p| !self.player_reg.verify_identifier(p))
        {
            // Saftey check, we already checked that all the identifiers correspond to a player
            let ids: Vec<PlayerId> = idents
                .into_iter()
                .map(|p| self.player_reg.get_player_id(&p).unwrap())
                .collect();
            let r_id = self.round_reg.create_round();
            for id in ids {
                let _ = self.round_reg.add_player_to_round(&r_id, id);
            }
            Ok(OpData::CreateRound(r_id))
        } else {
            Err(TournamentError::PlayerLookup)
        }
    }

    pub(crate) fn cut_to_top(&mut self, len: usize) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let player_iter = self
            .get_standings()
            .scores
            .into_iter()
            .skip(len)
            .map(|(id, _)| PlayerIdentifier::Id(id));
        for id in player_iter {
            // This result doesn't matter
            let _ = self.drop_player(&id);
        }
        Ok(OpData::Nothing)
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
    pub fn ready_player(&mut self, id: PlayerId) {
        match self {
            Self::Swiss(sys) => sys.ready_player(id),
            Self::Fluid(sys) => sys.ready_player(id),
        }
    }

    pub fn ready_to_pair(&self, plyr_reg: &PlayerRegistry, rnd_reg: &RoundRegistry) -> bool {
        match self {
            Self::Swiss(sys) => sys.ready_to_pair(plyr_reg, rnd_reg),
            Self::Fluid(sys) => sys.ready_to_pair(),
        }
    }

    pub fn pair<S>(
        &mut self,
        plyr_reg: &PlayerRegistry,
        rnd_reg: &RoundRegistry,
        standings: Standings<S>,
    ) -> Option<Pairings>
    where
        S: Score,
    {
        match self {
            Self::Swiss(sys) => sys.pair(plyr_reg, rnd_reg, standings),
            Self::Fluid(sys) => sys.pair(plyr_reg, rnd_reg),
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

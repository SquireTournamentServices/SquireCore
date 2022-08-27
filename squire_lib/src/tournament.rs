use std::{collections::HashMap, time::Duration};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// use mtgjson::model::deck::Deck;

use crate::{
    accounts::SquireAccount,
    admin::{Admin, Judge, TournOfficialId},
    error::TournamentError,
    identifiers::{AdminId, JudgeId, PlayerId, PlayerIdentifier, RoundIdentifier},
    operations::{OpData, OpResult, TournOp},
    pairings::{PairingStyle, PairingSystem},
    player::{Deck, Player, PlayerStatus},
    player_registry::PlayerRegistry,
    round::{Round, RoundResult},
    round_registry::RoundRegistry,
    scoring::Standings,
    settings::{self, TournamentSetting},
    standard_scoring::{StandardScore, StandardScoring},
};

pub use crate::identifiers::{TournamentId, TournamentIdentifier};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
/// An enum that encode the initial values of a tournament
pub enum TournamentPreset {
    /// The tournament will have a swiss pairing system and a standard scoring system
    Swiss,
    /// The tournament will have a fluid pairing system and a standard scoring system
    Fluid,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// An enum that encodes all the possible scoring systems a tournament can have.
/// (So many, much wow)
pub enum ScoringSystem {
    /// The tournament has a standard scoring system
    Standard(StandardScoring),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
/// An enum that encodes all the statuses of a tournament
pub enum TournamentStatus {
    /// The tournament can not create rounds
    Planned,
    /// All functionalities are unlocked
    Started,
    /// All functionalities except status changes are locked
    Frozen,
    /// The tournament is over after starting
    Ended,
    /// The tournament is over and was never started
    Cancelled,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// The core tournament structure
pub struct Tournament {
    /// The tournament's id
    pub id: TournamentId,
    /// The tournament's name
    pub name: String,
    /// Whether or not the tournament will assign table numbers
    pub use_table_number: bool,
    /// The format the tournament will be played in (meta data)
    pub format: String,
    /// The number of players in a round
    pub game_size: u8,
    /// The minimum number of decks a player needs
    pub min_deck_count: u8,
    /// The maximum number of decks a player can have
    pub max_deck_count: u8,
    /// The system for tracking players, their reg status, etc
    pub player_reg: PlayerRegistry,
    /// The system for creating and tracking rounds
    pub round_reg: RoundRegistry,
    /// The pairing system used to pair players
    pub pairing_sys: PairingSystem,
    /// The scoring system used to rank players
    pub scoring_sys: ScoringSystem,
    /// Whether or not new players can sign up for the tournament
    pub reg_open: bool,
    /// Whether or not a player must check in after signing up
    pub require_check_in: bool,
    /// Whether or not deck registration is required
    pub require_deck_reg: bool,
    /// The status of the tournament
    pub status: TournamentStatus,
    /// The set of judges for the tournament
    pub judges: HashMap<JudgeId, Judge>,
    /// The set of admins for the tournament
    pub admins: HashMap<AdminId, Admin>,
}

impl Tournament {
    /// Creates a new tournament from the defaults established by the given preset
    pub fn from_preset(name: String, preset: TournamentPreset, format: String) -> Self {
        Tournament {
            id: TournamentId::new(Uuid::new_v4()),
            name,
            use_table_number: true,
            format,
            game_size: 2,
            min_deck_count: 1,
            max_deck_count: 2,
            player_reg: PlayerRegistry::new(),
            round_reg: RoundRegistry::new(0, Duration::from_secs(3000)),
            pairing_sys: PairingSystem::new(preset),
            scoring_sys: scoring_system_factory(preset),
            reg_open: true,
            require_check_in: false,
            require_deck_reg: false,
            status: TournamentStatus::Planned,
            judges: HashMap::new(),
            admins: HashMap::new(),
        }
    }

    /// Applies a tournament operation to the tournament
    pub fn apply_op(&mut self, op: TournOp) -> OpResult {
        use TournOp::*;
        match op {
            Create(..) => OpResult::Ok(OpData::Nothing),
            // Player ops
            CheckIn(p_ident) => self.check_in(&p_ident),
            RegisterPlayer(account) => self.register_player(account),
            RecordResult(r_ident, result) => self.record_result(&r_ident, result),
            ConfirmResult(p_ident) => self.confirm_round(&p_ident),
            DropPlayer(p_ident) => self.drop_player(&p_ident),
            AddDeck(p_ident, name, deck) => self.player_add_deck(&p_ident, name, deck),
            RemoveDeck(p_ident, name) => self.remove_player_deck(&p_ident, name),
            SetGamerTag(p_ident, tag) => self.player_set_game_name(&p_ident, tag),
            ReadyPlayer(p_ident) => self.ready_player(&p_ident),
            UnReadyPlayer(p_ident) => self.unready_player(&p_ident),
            // Judge/Admin ops
            AdminDropPlayer(id, p_ident) => self.admin_drop_player(id, &p_ident),
            RemoveRound(id, r_ident) => self.remove_round(id, &r_ident),
            UpdateReg(id, b) => self.update_reg(id, b),
            Start(id) => self.start(id),
            Freeze(id) => self.freeze(id),
            Thaw(id) => self.thaw(id),
            End(id) => self.end(id),
            Cancel(id) => self.cancel(id),
            UpdateTournSetting(id, setting) => self.update_setting(id, setting),
            GiveBye(id, p_ident) => self.give_bye(id, &p_ident),
            CreateRound(id, p_idents) => self.create_round(id, p_idents),
            PairRound(id) => self.pair(id),
            TimeExtension(id, rnd, ext) => self.give_time_extension(id, &rnd, ext),
            Cut(id, n) => self.cut_to_top(id, n),
            PruneDecks(id) => self.prune_decks(id),
            PrunePlayers(id) => self.prune_players(id),
            AdminRegisterPlayer(id, account) => self.admin_register_player(id, account),
            RegisterGuest(id, name) => self.register_guest(id, name),
            RegisterJudge(id, account) => self.register_judge(id, account),
            RegisterAdmin(id, account) => self.register_admin(id, account),
            AdminRecordResult(id, rnd, result) => self.admin_record_result(id, rnd, result),
            AdminConfirmResult(id, ident) => self.admin_confirm_result(id, ident),
            AdminAddDeck(id, plyr, name, deck) => self.admin_add_deck(id, plyr, name, deck),
            AdminReadyPlayer(id, ident) => self.admin_ready_player(id, ident),
            AdminOverwriteResult(id, rnd, result) => self.admin_overwrite_result(id, rnd, result),
            AdminUnReadyPlayer(id, ident) => self.admin_unready_player(id, ident),
            //ImportPlayer(plyr) => self.import_player(plyr),
            //ImportRound(rnd) => self.import_round(rnd),
        }
    }

    /// Calculates if the tournament is planned
    pub fn is_planned(&self) -> bool {
        self.status == TournamentStatus::Planned
    }

    /// Calculates if the tournament is frozen
    pub fn is_frozen(&self) -> bool {
        self.status == TournamentStatus::Frozen
    }

    /// Calculates if the tournament is active
    pub fn is_active(&self) -> bool {
        self.status == TournamentStatus::Started
    }

    /// Calculates if the tournament is over
    pub fn is_dead(&self) -> bool {
        self.status == TournamentStatus::Ended || self.status == TournamentStatus::Cancelled
    }

    /// Calculates if someone is a judge
    pub fn is_judge(&self, id: &JudgeId) -> bool {
        self.judges.contains_key(id)
    }

    /// Calculates if someone is a tournament admin
    pub fn is_admin(&self, id: &AdminId) -> bool {
        self.admins.contains_key(id)
    }

    /// Calculates if someone is a tournament official
    pub fn is_official(&self, id: &TournOfficialId) -> bool {
        match id {
            TournOfficialId::Judge(id) => self.is_judge(id),
            TournOfficialId::Admin(id) => self.is_admin(id),
        }
    }

    /// Gets a copy of a player's registration data
    /// NOTE: This does not include their round data
    pub fn get_player(&self, ident: &PlayerIdentifier) -> Result<Player, TournamentError> {
        match self.player_reg.get_player(ident) {
            Some(plyr) => Ok(plyr.clone()),
            None => Err(TournamentError::PlayerLookup),
        }
    }

    /// Gets a copy of a round's data
    pub fn get_round(&self, ident: &RoundIdentifier) -> Result<Round, TournamentError> {
        match self.round_reg.get_round(ident) {
            Some(rnd) => Ok(rnd.clone()),
            None => Err(TournamentError::RoundLookup),
        }
    }

    /// Gets all the rounds a player is in
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

    /// Gets a copy of a specific deck from a player
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

    /// Gets a copy of all the decks a player has registered
    pub fn get_player_decks(
        &self,
        ident: &PlayerIdentifier,
    ) -> Result<HashMap<String, Deck>, TournamentError> {
        let plyr = self
            .player_reg
            .get_player(ident)
            .ok_or(TournamentError::PlayerLookup)?;
        Ok(plyr.decks.clone())
    }

    /// Gets the current standing of the tournament
    pub fn get_standings(&self) -> Standings<StandardScore> {
        self.scoring_sys
            .get_standings(&self.player_reg, &self.round_reg)
    }

    /// Removes excess decks for players (as defined by `max_deck_count`). The newest decks are
    /// kept.
    pub(crate) fn prune_decks(&mut self, _: AdminId) -> OpResult {
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

    /// Removes players from the tournament that did not complete registration.
    /// This include players that did not submit enough decks (defined by `require_deck_reg` and
    /// `min_deck_count`) and that didn't check in (defined by `require_check_in`).
    pub(crate) fn prune_players(&mut self, _: AdminId) -> OpResult {
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

    /// Adds a time extension to a round
    pub(crate) fn give_time_extension(
        &mut self,
        _: TournOfficialId,
        rnd: &RoundIdentifier,
        ext: Duration,
    ) -> OpResult {
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

    /// Checks in a player for the tournament.
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

    /// Attempts to create the next set of rounds for the tournament
    pub(crate) fn pair(&mut self, _: AdminId) -> OpResult {
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
            if let PairingStyle::Swiss(_) = &self.pairing_sys.style {
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

    /// Makes a round irrelevant to the tournament.
    /// NOTE: The round will still exist but will have a "dead" status and will be ignored by the
    /// tournament.
    pub(crate) fn remove_round(&mut self, _: AdminId, ident: &RoundIdentifier) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        self.round_reg.kill_round(ident)?;
        Ok(OpData::Nothing)
    }

    /// Updates a single tournament setting
    pub(crate) fn update_setting(&mut self, _: AdminId, setting: TournamentSetting) -> OpResult {
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
                if c > self.max_deck_count {
                    return Err(TournamentError::InvalidDeckCount);
                }
                self.min_deck_count = c;
            }
            MaxDeckCount(c) => {
                if c < self.min_deck_count {
                    return Err(TournamentError::InvalidDeckCount);
                }
                self.max_deck_count = c;
            }
            RequireCheckIn(b) => {
                self.require_check_in = b;
            }
            RequireDeckReg(b) => {
                self.require_deck_reg = b;
            }
            PairingSetting(setting) => {
                self.pairing_sys.update_setting(setting)?;
            }
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

    /// Changes the registration status
    pub(crate) fn update_reg(&mut self, _: AdminId, reg_status: bool) -> OpResult {
        if self.is_frozen() || self.is_dead() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        self.reg_open = reg_status;
        Ok(OpData::Nothing)
    }

    /// Sets the tournament status to `Active`.
    pub(crate) fn start(&mut self, _: AdminId) -> OpResult {
        if !self.is_planned() {
            Err(TournamentError::IncorrectStatus(self.status))
        } else {
            self.reg_open = false;
            self.status = TournamentStatus::Started;
            Ok(OpData::Nothing)
        }
    }

    /// Sets the tournament status to `Frozen`.
    pub(crate) fn freeze(&mut self, _: AdminId) -> OpResult {
        if !self.is_active() {
            Err(TournamentError::IncorrectStatus(self.status))
        } else {
            self.reg_open = false;
            self.status = TournamentStatus::Frozen;
            Ok(OpData::Nothing)
        }
    }

    /// Sets the tournament status to `Active` only if the current status is `Frozen`
    pub(crate) fn thaw(&mut self, _: AdminId) -> OpResult {
        if !self.is_frozen() {
            Err(TournamentError::IncorrectStatus(self.status))
        } else {
            self.status = TournamentStatus::Started;
            Ok(OpData::Nothing)
        }
    }

    /// Sets the tournament status to `Ended`.
    pub(crate) fn end(&mut self, _: AdminId) -> OpResult {
        if !self.is_active() {
            Err(TournamentError::IncorrectStatus(self.status))
        } else {
            self.reg_open = false;
            self.status = TournamentStatus::Ended;
            Ok(OpData::Nothing)
        }
    }

    /// Sets the tournament status to `Cancelled`.
    pub(crate) fn cancel(&mut self, _: AdminId) -> OpResult {
        if !self.is_active() {
            Err(TournamentError::IncorrectStatus(self.status))
        } else {
            self.reg_open = false;
            self.status = TournamentStatus::Cancelled;
            Ok(OpData::Nothing)
        }
    }

    /// Adds a player to the tournament
    pub(crate) fn register_player(&mut self, account: SquireAccount) -> OpResult {
        if !(self.is_active() || self.is_planned()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if !self.reg_open {
            return Err(TournamentError::RegClosed);
        }
        let id = self.player_reg.add_player(account)?;
        Ok(OpData::RegisterPlayer(PlayerIdentifier::Id(id)))
    }

    /// Records part of the result of a round
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

    /// A player confirms the round record
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

    /// Dropps a player from the tournament
    pub(crate) fn drop_player(&mut self, ident: &PlayerIdentifier) -> OpResult {
        if self.is_dead() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        self.player_reg
            .drop_player(ident)
            .ok_or(TournamentError::PlayerLookup)?;
        let id = self.player_reg.get_player_id(ident).unwrap();
        for rnd in self.round_reg.get_player_active_rounds(&id) {
            rnd.remove_player(id);
        }
        Ok(OpData::Nothing)
    }

    /// An admin drops a player
    pub(crate) fn admin_drop_player(&mut self, _: AdminId, ident: &PlayerIdentifier) -> OpResult {
        if self.is_dead() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        self.player_reg
            .drop_player(ident)
            .ok_or(TournamentError::PlayerLookup)?;
        let id = self.player_reg.get_player_id(ident).unwrap();
        for rnd in self.round_reg.get_player_active_rounds(&id) {
            rnd.remove_player(id);
        }
        Ok(OpData::Nothing)
    }

    /// Adds a deck to a player's registration data
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

    /// Removes a player's deck from their registration data
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

    /// Sets a player's gamer tag
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
        plyr.game_name = Some(name);
        Ok(OpData::Nothing)
    }

    /// Readies a player to play in their next round
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
            self.pairing_sys.ready_player(plyr.id);
            should_pair = match &self.pairing_sys.style {
                PairingStyle::Fluid(_) => self
                    .pairing_sys
                    .ready_to_pair(&self.player_reg, &self.round_reg),
                PairingStyle::Swiss(_) => false,
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

    /// Marks a player has unready to player in their next round
    pub(crate) fn unready_player(&mut self, plyr: &PlayerIdentifier) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let plyr = self
            .player_reg
            .get_player_id(plyr)
            .ok_or(TournamentError::PlayerLookup)?;
        self.pairing_sys.unready_player(plyr);
        Ok(OpData::Nothing)
    }

    /// Gives a player a bye
    pub(crate) fn give_bye(&mut self, _: AdminId, ident: &PlayerIdentifier) -> OpResult {
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

    /// Creates a new round from a list of players
    pub(crate) fn create_round(&mut self, _: AdminId, idents: Vec<PlayerIdentifier>) -> OpResult {
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

    /// Drops all by the top N players (by standings)
    pub(crate) fn cut_to_top(&mut self, _: AdminId, len: usize) -> OpResult {
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

    pub(crate) fn admin_register_player(
        &mut self,
        id: TournOfficialId,
        account: SquireAccount,
    ) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if !self.is_official(&id) {
            return Err(TournamentError::OfficalLookup);
        }
        let id = self.player_reg.add_player(account)?;
        Ok(OpData::RegisterPlayer(PlayerIdentifier::Id(id)))
    }

    pub(crate) fn register_guest(&mut self, id: TournOfficialId, name: String) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if !self.is_official(&id) {
            return Err(TournamentError::OfficalLookup);
        }
        let id = self.player_reg.add_guest(name)?;
        Ok(OpData::RegisterPlayer(PlayerIdentifier::Id(id)))
    }

    pub(crate) fn register_judge(&mut self, id: AdminId, account: SquireAccount) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if !self.is_admin(&id) {
            return Err(TournamentError::OfficalLookup);
        }
        let judge = Judge::new(account);
        self.judges.insert(judge.id, judge.clone());
        Ok(OpData::RegisterJudge(judge))
    }

    pub(crate) fn register_admin(&mut self, id: AdminId, account: SquireAccount) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if !self.is_admin(&id) {
            return Err(TournamentError::OfficalLookup);
        }
        let admin = Admin::new(account);
        self.admins.insert(admin.id, admin.clone());
        Ok(OpData::RegisterAdmin(admin))
    }

    pub(crate) fn admin_add_deck(
        &mut self,
        id: TournOfficialId,
        ident: PlayerIdentifier,
        name: String,
        deck: Deck,
    ) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if !self.is_official(&id) {
            return Err(TournamentError::OfficalLookup);
        }
        let plyr = self
            .player_reg
            .get_mut_player(&ident)
            .ok_or(TournamentError::PlayerLookup)?;
        plyr.add_deck(name, deck);
        Ok(OpData::Nothing)
    }

    pub(crate) fn admin_record_result(
        &mut self,
        id: TournOfficialId,
        ident: RoundIdentifier,
        result: RoundResult,
    ) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if !self.is_official(&id) {
            return Err(TournamentError::OfficalLookup);
        }
        let round = self
            .round_reg
            .get_mut_round(&ident)
            .ok_or(TournamentError::RoundLookup)?;
        round.record_result(result)?;
        Ok(OpData::Nothing)
    }

    pub(crate) fn admin_confirm_result(
        &mut self,
        id: TournOfficialId,
        ident: PlayerIdentifier,
    ) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if !self.is_official(&id) {
            return Err(TournamentError::OfficalLookup);
        }
        let id = self
            .player_reg
            .get_player_id(&ident)
            .ok_or(TournamentError::PlayerLookup)?;
        let round = self.round_reg.get_player_active_round(&id)?;
        let status = round.confirm_round(id)?;
        Ok(OpData::ConfirmResult(status))
    }

    pub(crate) fn admin_ready_player(
        &mut self,
        id: TournOfficialId,
        ident: PlayerIdentifier,
    ) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if !self.is_official(&id) {
            return Err(TournamentError::OfficalLookup);
        }
        let plyr = self
            .player_reg
            .get_player(&ident)
            .ok_or(TournamentError::PlayerLookup)?;
        let mut should_pair = false;
        if plyr.can_play() {
            self.pairing_sys.ready_player(plyr.id);
            should_pair = match &self.pairing_sys.style {
                PairingStyle::Fluid(_) => self
                    .pairing_sys
                    .ready_to_pair(&self.player_reg, &self.round_reg),
                PairingStyle::Swiss(_) => false,
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

    pub(crate) fn admin_overwrite_result(
        &mut self,
        id: AdminId,
        ident: RoundIdentifier,
        result: RoundResult,
    ) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if !self.is_admin(&id) {
            return Err(TournamentError::OfficalLookup);
        }
        let round = self
            .round_reg
            .get_mut_round(&ident)
            .ok_or(TournamentError::RoundLookup)?;
        round.record_result(result)?;
        Ok(OpData::Nothing)
    }

    pub(crate) fn admin_unready_player(
        &mut self,
        id: TournOfficialId,
        ident: PlayerIdentifier,
    ) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if !self.is_official(&id) {
            return Err(TournamentError::OfficalLookup);
        }
        let plyr = self
            .player_reg
            .get_player_id(&ident)
            .ok_or(TournamentError::PlayerLookup)?;
        self.pairing_sys.unready_player(plyr);
        Ok(OpData::Nothing)
    }
}

impl ScoringSystem {
    /// Gets the current standings of all players
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

impl From<StandardScoring> for ScoringSystem {
    fn from(other: StandardScoring) -> Self {
        Self::Standard(other)
    }
}

/// Creates scoring systems from a tournament preset
pub fn scoring_system_factory(preset: TournamentPreset) -> ScoringSystem {
    use TournamentPreset::*;
    match preset {
        Swiss => StandardScoring::new().into(),
        Fluid => StandardScoring::new().into(),
    }
}

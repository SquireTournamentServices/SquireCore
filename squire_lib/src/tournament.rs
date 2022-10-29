use std::{collections::HashMap, fmt::Display, time::Duration};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// use mtgjson::model::deck::Deck;

use crate::{
    accounts::SquireAccount,
    admin::{Admin, Judge, TournOfficialId},
    error::TournamentError,
    identifiers::{AdminId, JudgeId, PlayerId, PlayerIdentifier, RoundId, RoundIdentifier},
    operations::{AdminOp, JudgeOp, OpData, OpResult, PlayerOp, TournOp},
    pairings::{PairingStyle, PairingSystem},
    players::{Deck, Player, PlayerRegistry, PlayerStatus},
    rounds::{Round, RoundRegistry, RoundResult},
    scoring::{ScoringSystem, StandardScore, StandardScoring, Standings},
    settings::{self, TournamentSetting},
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
            RegisterPlayer(account) => self.register_player(account),
            PlayerOp(p_id, op) => self.apply_player_op(p_id, op),
            JudgeOp(ta_id, op) => self.apply_judge_op(ta_id, op),
            AdminOp(a_id, op) => self.apply_admin_op(a_id, op),
        }
    }

    fn apply_player_op(&mut self, p_id: PlayerId, op: PlayerOp) -> OpResult {
        match op {
            PlayerOp::CheckIn => self.check_in(p_id),
            PlayerOp::RecordResult(result) => self.record_result(&p_id, result),
            PlayerOp::ConfirmResult => self.confirm_round(p_id),
            PlayerOp::DropPlayer => self.drop_player(p_id),
            PlayerOp::AddDeck(name, deck) => self.player_add_deck(&p_id, name, deck),
            PlayerOp::RemoveDeck(name) => self.remove_player_deck(&p_id, name),
            PlayerOp::SetGamerTag(tag) => self.player_set_game_name(&p_id, tag),
            PlayerOp::ReadyPlayer => self.ready_player(&p_id),
            PlayerOp::UnReadyPlayer => self.unready_player(p_id),
        }
    }

    fn apply_judge_op(&mut self, ta_id: TournOfficialId, op: JudgeOp) -> OpResult {
        if !self.is_official(&ta_id) {
            return OpResult::Err(TournamentError::OfficalLookup);
        }
        match op {
            JudgeOp::AdminRegisterPlayer(account) => self.admin_register_player(account),
            JudgeOp::RegisterGuest(name) => self.register_guest(name),
            JudgeOp::AdminAddDeck(plyr, name, deck) => self.admin_add_deck(plyr, name, deck),
            JudgeOp::AdminRemoveDeck(plyr, name) => self.admin_remove_deck(plyr, name),
            JudgeOp::AdminReadyPlayer(p_id) => self.admin_ready_player(p_id),
            JudgeOp::AdminUnReadyPlayer(p_id) => self.admin_unready_player(p_id),
            JudgeOp::AdminRecordResult(rnd, result) => self.admin_record_result(rnd, result),
            JudgeOp::AdminConfirmResult(r_id, p_id) => self.admin_confirm_result(r_id, p_id),
            JudgeOp::TimeExtension(rnd, ext) => self.give_time_extension(&rnd, ext),
        }
    }

    fn apply_admin_op(&mut self, a_id: AdminId, op: AdminOp) -> OpResult {
        if !self.is_admin(&a_id) {
            return OpResult::Err(TournamentError::OfficalLookup);
        }
        match op {
            AdminOp::RemoveRound(r_id) => self.remove_round(&r_id),
            AdminOp::AdminOverwriteResult(rnd, result) => self.admin_overwrite_result(rnd, result),
            AdminOp::AdminDropPlayer(p_id) => self.admin_drop_player(p_id),
            AdminOp::UpdateReg(b) => self.update_reg(b),
            AdminOp::Start => self.start(),
            AdminOp::Freeze => self.freeze(),
            AdminOp::Thaw => self.thaw(),
            AdminOp::End => self.end(),
            AdminOp::Cancel => self.cancel(),
            AdminOp::UpdateTournSetting(setting) => self.update_setting(setting),
            AdminOp::GiveBye(p_id) => self.give_bye(p_id),
            AdminOp::CreateRound(p_ids) => self.create_round(p_ids),
            AdminOp::PairRound => self.pair(),
            AdminOp::Cut(n) => self.cut_to_top(n),
            AdminOp::PruneDecks => self.prune_decks(),
            AdminOp::PrunePlayers => self.prune_players(),
            AdminOp::RegisterJudge(account) => self.register_judge(account),
            AdminOp::RegisterAdmin(account) => self.register_admin(account),
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
    pub fn get_player_id(&self, ident: &PlayerIdentifier) -> Result<PlayerId, TournamentError> {
        match ident {
            PlayerIdentifier::Id(id) => self
                .player_reg
                .is_registered(id)
                .then(|| *id)
                .ok_or_else(|| TournamentError::PlayerLookup),
            PlayerIdentifier::Name(name) => self.player_reg.get_player_id(name),
        }
    }

    /// Gets a copy of a player's registration data
    /// NOTE: This does not include their round data
    pub fn get_player(&self, ident: &PlayerIdentifier) -> Result<&Player, TournamentError> {
        match ident {
            PlayerIdentifier::Id(id) => self.player_reg.get_player(id),
            PlayerIdentifier::Name(name) => self.player_reg.get_by_name(name),
        }
    }

    /// Gets a copy of a player's registration data
    /// NOTE: This does not include their round data
    pub fn get_round_id(&self, ident: &RoundIdentifier) -> Result<RoundId, TournamentError> {
        match ident {
            RoundIdentifier::Id(id) => self
                .round_reg
                .validate_id(id)
                .then(|| *id)
                .ok_or_else(|| TournamentError::RoundLookup),
            RoundIdentifier::Number(num) => self.round_reg.get_round_id(num),
        }
    }

    /// Gets a copy of a round's data
    pub fn get_round(&self, ident: &RoundIdentifier) -> Result<&Round, TournamentError> {
        match ident {
            RoundIdentifier::Id(id) => self.round_reg.get_round(id),
            RoundIdentifier::Number(num) => self.round_reg.get_by_number(num),
        }
    }

    /// Gets all the rounds a player is in
    pub fn get_player_rounds(
        &self,
        ident: &PlayerIdentifier,
    ) -> Result<Vec<&Round>, TournamentError> {
        let id = match ident {
            PlayerIdentifier::Id(id) => *id,
            PlayerIdentifier::Name(name) => self.player_reg.get_player_id(name)?,
        };
        Ok(self
            .round_reg
            .rounds
            .iter()
            .filter_map(|(_, r)| r.players.get(&id).map(|_| r))
            .collect())
    }

    /// Gets a copy of a specific deck from a player
    pub fn get_player_deck(
        &self,
        ident: &PlayerIdentifier,
        name: &String,
    ) -> Result<&Deck, TournamentError> {
        self.get_player(ident)?
            .get_deck(name)
            .ok_or_else(|| TournamentError::DeckLookup)
    }

    /// Gets a copy of all the decks a player has registered
    pub fn get_player_decks(
        &self,
        ident: &PlayerIdentifier,
    ) -> Result<&HashMap<String, Deck>, TournamentError> {
        self.get_player(ident).map(|p| &p.decks)
    }

    /// Gets the current standing of the tournament
    pub fn get_standings(&self) -> Standings<StandardScore> {
        self.scoring_sys
            .get_standings(&self.player_reg, &self.round_reg)
    }

    /// Removes excess decks for players (as defined by `max_deck_count`). The newest decks are
    /// kept.
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

    /// Removes players from the tournament that did not complete registration.
    /// This include players that did not submit enough decks (defined by `require_deck_reg` and
    /// `min_deck_count`) and that didn't check in (defined by `require_check_in`).
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

    /// Adds a time extension to a round
    pub(crate) fn give_time_extension(&mut self, rnd: &RoundId, ext: Duration) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let round = self.round_reg.get_mut_round(rnd)?;
        round.extension += ext;
        Ok(OpData::Nothing)
    }

    /// Checks in a player for the tournament.
    pub(crate) fn check_in(&mut self, id: PlayerId) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if self.is_planned() {
            self.player_reg.check_in(id);
            Ok(OpData::Nothing)
        } else {
            Err(TournamentError::IncorrectStatus(self.status))
        }
    }

    /// Attempts to create the next set of rounds for the tournament
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
            if let PairingStyle::Swiss(_) = &self.pairing_sys.style {
                for plyr in pairings.rejected {
                    let r_id = self.round_reg.create_round();
                    let ident = r_id.into();
                    let rnd = self.round_reg.get_mut_round(&ident).unwrap();
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
    pub(crate) fn remove_round(&mut self, ident: &RoundId) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        self.round_reg.kill_round(ident)?;
        Ok(OpData::Nothing)
    }

    /// Updates a single tournament setting
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
            RoundLength(dur) => {
                self.round_reg.length = dur;
            }
            PairingSetting(setting) => {
                self.pairing_sys.update_setting(setting)?;
            }
            ScoringSetting(setting) => match setting {
                settings::ScoringSetting::Standard(s) => {
                    #[allow(irrefutable_let_patterns)]
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
    pub(crate) fn update_reg(&mut self, reg_status: bool) -> OpResult {
        if self.is_frozen() || self.is_dead() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        self.reg_open = reg_status;
        Ok(OpData::Nothing)
    }

    /// Sets the tournament status to `Active`.
    pub(crate) fn start(&mut self) -> OpResult {
        if !self.is_planned() {
            Err(TournamentError::IncorrectStatus(self.status))
        } else {
            self.reg_open = false;
            self.status = TournamentStatus::Started;
            Ok(OpData::Nothing)
        }
    }

    /// Sets the tournament status to `Frozen`.
    pub(crate) fn freeze(&mut self) -> OpResult {
        if !self.is_active() {
            Err(TournamentError::IncorrectStatus(self.status))
        } else {
            self.reg_open = false;
            self.status = TournamentStatus::Frozen;
            Ok(OpData::Nothing)
        }
    }

    /// Sets the tournament status to `Active` only if the current status is `Frozen`
    pub(crate) fn thaw(&mut self) -> OpResult {
        if !self.is_frozen() {
            Err(TournamentError::IncorrectStatus(self.status))
        } else {
            self.status = TournamentStatus::Started;
            Ok(OpData::Nothing)
        }
    }

    /// Sets the tournament status to `Ended`.
    pub(crate) fn end(&mut self) -> OpResult {
        if !self.is_active() {
            Err(TournamentError::IncorrectStatus(self.status))
        } else {
            self.reg_open = false;
            self.status = TournamentStatus::Ended;
            Ok(OpData::Nothing)
        }
    }

    /// Sets the tournament status to `Cancelled`.
    pub(crate) fn cancel(&mut self) -> OpResult {
        if self.is_planned() {
            self.reg_open = false;
            self.status = TournamentStatus::Cancelled;
            Ok(OpData::Nothing)
        } else {
            Err(TournamentError::IncorrectStatus(self.status))
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
        Ok(OpData::RegisterPlayer(id))
    }

    /// Records part of the result of a round
    pub(crate) fn record_result(&mut self, id: &PlayerId, result: RoundResult) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let round = self.round_reg.get_player_active_round(id)?;
        round.record_result(result)?;
        Ok(OpData::Nothing)
    }

    /// A player confirms the round record
    pub(crate) fn confirm_round(&mut self, id: PlayerId) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let round = self.round_reg.get_player_active_round(&id)?;
        let status = round.confirm_round(id)?;
        Ok(OpData::ConfirmResult(round.id, status))
    }

    /// Dropps a player from the tournament
    pub(crate) fn drop_player(&mut self, id: PlayerId) -> OpResult {
        if self.is_dead() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        self.player_reg.drop_player(&id)?;
        for rnd in self.round_reg.get_player_active_rounds(&id) {
            rnd.remove_player(id);
        }
        Ok(OpData::Nothing)
    }

    /// An admin drops a player
    pub(crate) fn admin_drop_player(&mut self, id: PlayerId) -> OpResult {
        if self.is_dead() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        self.player_reg.drop_player(&id)?;
        for rnd in self.round_reg.get_player_active_rounds(&id) {
            rnd.remove_player(id);
        }
        Ok(OpData::Nothing)
    }

    /// Adds a deck to a player's registration data
    pub(crate) fn player_add_deck(
        &mut self,
        ident: &PlayerId,
        name: String,
        deck: Deck,
    ) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if !self.reg_open {
            return Err(TournamentError::RegClosed);
        }
        let plyr = self.player_reg.get_mut_player(ident)?;
        plyr.add_deck(name, deck);
        Ok(OpData::Nothing)
    }

    /// Removes a player's deck from their registration data
    pub(crate) fn remove_player_deck(&mut self, ident: &PlayerId, name: String) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let plyr = self.player_reg.get_mut_player(ident)?;
        plyr.remove_deck(name)?;
        Ok(OpData::Nothing)
    }

    /// Sets a player's gamer tag
    pub(crate) fn player_set_game_name(&mut self, ident: &PlayerId, name: String) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let plyr = self.player_reg.get_mut_player(ident)?;
        plyr.game_name = Some(name);
        Ok(OpData::Nothing)
    }

    /// Readies a player to play in their next round
    pub(crate) fn ready_player(&mut self, ident: &PlayerId) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let plyr = self.player_reg.get_player(ident)?;
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
    pub(crate) fn unready_player(&mut self, plyr: PlayerId) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        self.pairing_sys.unready_player(plyr);
        Ok(OpData::Nothing)
    }

    /// Gives a player a bye
    pub(crate) fn give_bye(&mut self, id: PlayerId) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let r_id = self.round_reg.create_round();
        let _ = self.round_reg.add_player_to_round(&r_id, id);
        // Saftey check: This should never return an Err as we just created the round and gave it a
        // single player
        let round = self.round_reg.get_mut_round(&r_id).unwrap();
        round.record_bye()?;
        Ok(OpData::GiveBye(r_id))
    }

    /// Creates a new round from a list of players
    pub fn create_round(&mut self, ids: Vec<PlayerId>) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if ids.len() == self.pairing_sys.match_size as usize
            && ids.iter().all(|p| self.player_reg.is_registered(p))
        {
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
    pub(crate) fn cut_to_top(&mut self, len: usize) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let player_iter = self
            .get_standings()
            .scores
            .into_iter()
            .skip(len)
            .map(|(id, _)| id);
        for id in player_iter {
            let _ = self.drop_player(id);
        }
        Ok(OpData::Nothing)
    }

    fn admin_register_player(&mut self, account: SquireAccount) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        Ok(OpData::RegisterPlayer(self.player_reg.add_player(account)?))
    }

    fn register_guest(&mut self, name: String) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        Ok(OpData::RegisterPlayer(self.player_reg.add_guest(name)?))
    }

    fn register_judge(&mut self, account: SquireAccount) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let judge = Judge::new(account);
        self.judges.insert(judge.id, judge.clone());
        Ok(OpData::RegisterJudge(judge))
    }

    fn register_admin(&mut self, account: SquireAccount) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let admin = Admin::new(account);
        self.admins.insert(admin.id, admin.clone());
        Ok(OpData::RegisterAdmin(admin))
    }

    fn admin_add_deck(&mut self, id: PlayerId, name: String, deck: Deck) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let plyr = self.player_reg.get_mut_player(&id)?;
        plyr.add_deck(name, deck);
        Ok(OpData::Nothing)
    }

    fn admin_remove_deck(&mut self, id: PlayerId, name: String) -> OpResult {
        if !(self.is_planned() || self.is_active()) {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        self.player_reg.get_mut_player(&id)?.remove_deck(name)?;
        Ok(OpData::Nothing)
    }

    fn admin_record_result(&mut self, id: RoundId, result: RoundResult) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        self.round_reg.get_mut_round(&id)?.record_result(result)?;
        Ok(OpData::Nothing)
    }

    fn admin_confirm_result(&mut self, r_id: RoundId, p_id: PlayerId) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let round = self.round_reg.get_mut_round(&r_id)?;
        let status = round.confirm_round(p_id)?;
        Ok(OpData::ConfirmResult(round.id, status))
    }

    fn admin_ready_player(&mut self, id: PlayerId) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let plyr = self.player_reg.get_player(&id)?;
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

    pub(crate) fn admin_overwrite_result(&mut self, id: RoundId, result: RoundResult) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        self.round_reg.get_mut_round(&id)?.record_result(result)?;
        Ok(OpData::Nothing)
    }

    pub(crate) fn admin_unready_player(&mut self, id: PlayerId) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        self.pairing_sys.unready_player(id);
        Ok(OpData::Nothing)
    }

    /// Counts players that are not fully checked in.
    /// First number is insufficient number of decks.
    /// Second number is not checked in.
    pub fn count_to_prune_players(&self) -> (usize, usize) {
        let mut digest = (0, 0);
        if self.require_deck_reg {
            digest.0 = self
                .player_reg
                .players
                .values()
                .filter(|p| p.decks.len() < self.min_deck_count as usize)
                .count();
        }
        if self.require_check_in {
            digest.1 = self.player_reg.len() - self.player_reg.check_ins.len();
        }
        digest
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

impl Display for TournamentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TournamentStatus::Planned => "Planned",
                TournamentStatus::Started => "Started",
                TournamentStatus::Frozen => "Frozen",
                TournamentStatus::Ended => "Ended",
                TournamentStatus::Cancelled => "Cancelled",
            }
        )
    }
}

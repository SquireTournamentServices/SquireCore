use std::{
    collections::HashMap,
    fmt::{Display, Write},
    time::Duration,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Seq};
use uuid::Uuid;

pub use crate::identifiers::{TournamentId, TournamentIdentifier};
use crate::{
    accounts::SquireAccount,
    admin::{Admin, Judge, TournOfficialId},
    error::TournamentError,
    identifiers::{AdminId, JudgeId, PlayerId, PlayerIdentifier, RoundId, RoundIdentifier},
    operations::{AdminOp, JudgeOp, OpData, OpResult, PlayerOp, TournOp},
    pairings::{PairingStyle, PairingSystem, Pairings},
    players::{Deck, Player, PlayerRegistry, PlayerStatus},
    rounds::{Round, RoundRegistry, RoundResult, RoundStatus},
    scoring::{ScoringSystem, StandardScore, Standings},
    settings::{GeneralSettingsTree, TournamentSetting, TournamentSettingsTree},
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(C)]
/// An enum that encode the initial values of a tournament
pub enum TournamentPreset {
    /// The tournament will have a swiss pairing system and a standard scoring system
    Swiss,
    /// The tournament will have a fluid pairing system and a standard scoring system
    Fluid,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
/// Container for all the information needed to start a tournament
pub struct TournamentSeed {
    /// The name of the to-be tournament
    pub name: String,
    /// The initial present of the to-be tournament
    pub preset: TournamentPreset,
    /// The initial format fo the to-be tournament
    pub format: String,
}

#[derive(
    Serialize, Deserialize, Default, Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
)]
#[repr(C)]
/// An enum that encodes all the statuses of a tournament
pub enum TournamentStatus {
    /// The tournament can not create rounds
    #[default]
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

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// The core tournament structure
pub struct Tournament {
    /// The tournament's id
    pub id: TournamentId,
    /// The tournament's name
    pub name: String,
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
    /// General settings for the tournament, including round length and whether or not to use table
    /// numbers
    #[serde(default)]
    pub settings: GeneralSettingsTree,
    /// The status of the tournament
    pub status: TournamentStatus,
    /// The set of judges for the tournament
    #[serde_as(as = "Seq<(_, _)>")]
    pub judges: HashMap<JudgeId, Judge>,
    /// The set of admins for the tournament
    #[serde_as(as = "Seq<(_, _)>")]
    pub admins: HashMap<AdminId, Admin>,
}

impl Tournament {
    /// Creates a new tournament from the defaults established by the given preset
    pub fn from_preset(
        name: String,
        preset: TournamentPreset,
        format: String,
    ) -> Result<Self, TournamentError> {
        // name validation
        if name.is_empty() || !name.is_ascii() {
            return Err(TournamentError::BadTournamentName);
        }

        Ok(Tournament {
            // TODO: This should be calculated from some salt and the name
            id: TournamentId::new(Uuid::new_v4()),
            name,
            settings: GeneralSettingsTree::with_format(format),
            player_reg: PlayerRegistry::new(),
            round_reg: RoundRegistry::new(0, Duration::from_secs(3000)),
            pairing_sys: PairingSystem::new(preset),
            scoring_sys: ScoringSystem::new(preset),
            reg_open: true,
            status: TournamentStatus::Planned,
            judges: HashMap::new(),
            admins: HashMap::new(),
        })
    }

    /// Applies a tournament operation to the tournament
    pub fn apply_op(&mut self, salt: DateTime<Utc>, op: TournOp) -> OpResult {
        use TournOp::*;
        match op {
            RegisterPlayer(account) => self.register_player(account),
            PlayerOp(p_id, op) => self.apply_player_op(salt, p_id, op),
            JudgeOp(ta_id, op) => self.apply_judge_op(salt, ta_id, op),
            AdminOp(a_id, op) => self.apply_admin_op(salt, a_id, op),
        }
    }

    fn apply_player_op(&mut self, salt: DateTime<Utc>, p_id: PlayerId, op: PlayerOp) -> OpResult {
        match op {
            PlayerOp::CheckIn => self.check_in(p_id),
            PlayerOp::RecordResult(r_id, result) => self.record_result(&r_id, result),
            PlayerOp::ConfirmResult(r_id) => self.confirm_round(r_id, p_id),
            PlayerOp::DropPlayer => self.drop_player(p_id),
            PlayerOp::AddDeck(name, deck) => self.player_add_deck(p_id, name, deck),
            PlayerOp::RemoveDeck(name) => self.remove_player_deck(&p_id, name),
            PlayerOp::SetGamerTag(tag) => self.player_set_game_name(&p_id, tag),
            PlayerOp::ReadyPlayer => self.ready_player(salt, &p_id),
            PlayerOp::UnReadyPlayer => self.unready_player(p_id),
        }
    }

    fn apply_judge_op(
        &mut self,
        salt: DateTime<Utc>,
        ta_id: TournOfficialId,
        op: JudgeOp,
    ) -> OpResult {
        if !self.is_official(&ta_id) {
            return OpResult::Err(TournamentError::OfficalLookup);
        }
        match op {
            JudgeOp::AdminRegisterPlayer(account) => self.admin_register_player(account),
            JudgeOp::RegisterGuest(name) => self.register_guest(salt, name),
            JudgeOp::ReRegisterGuest(name) => self.reregister_guest(name),
            JudgeOp::AdminAddDeck(plyr, name, deck) => self.admin_add_deck(plyr, name, deck),
            JudgeOp::AdminRemoveDeck(plyr, name) => self.admin_remove_deck(plyr, name),
            JudgeOp::AdminReadyPlayer(p_id) => self.admin_ready_player(salt, p_id),
            JudgeOp::AdminUnReadyPlayer(p_id) => self.admin_unready_player(p_id),
            JudgeOp::AdminRecordResult(rnd, result) => self.admin_record_result(rnd, result),
            JudgeOp::AdminConfirmResult(r_id, p_id) => self.admin_confirm_result(r_id, p_id),
            JudgeOp::TimeExtension(rnd, ext) => self.give_time_extension(&rnd, ext),
            JudgeOp::ConfirmRound(rnd) => self.confirm_single_round(&rnd),
        }
    }

    fn apply_admin_op(&mut self, salt: DateTime<Utc>, a_id: AdminId, op: AdminOp) -> OpResult {
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
            AdminOp::GiveBye(p_id) => self.give_bye(salt, p_id),
            AdminOp::CreateRound(p_ids) => self.create_round(salt, p_ids),
            AdminOp::PairRound(pairings) => self.pair(salt, pairings),
            AdminOp::Cut(n) => self.cut_to_top(n),
            AdminOp::PrunePlayers => self.prune_players(),
            AdminOp::RegisterJudge(account) => self.register_judge(account),
            AdminOp::RegisterAdmin(account) => self.register_admin(account),
            AdminOp::ConfirmAllRounds => self.confirm_all_rounds(),
        }
    }

    /// Calculates if the tournament is planned
    pub fn is_planned(&self) -> bool {
        self.status == TournamentStatus::Planned
    }

    /// Calculates if the tournament is planned
    pub fn is_ongoing(&self) -> bool {
        self.status == TournamentStatus::Planned || self.status == TournamentStatus::Started
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

    /// Calculates the number of players in the tournament, regardless of status
    pub fn get_player_count(&self) -> usize {
        self.player_reg.players.len()
    }

    /// Calculates the number of rounds in the tournament, regardless of status
    pub fn get_round_count(&self) -> usize {
        self.round_reg.rounds.len()
    }

    /// Gets a copy of a player's registration data
    /// NOTE: This does not include their round data
    pub fn get_player_id(&self, ident: &PlayerIdentifier) -> Result<PlayerId, TournamentError> {
        match ident {
            PlayerIdentifier::Id(id) => self
                .player_reg
                .is_registered(id)
                .then_some(*id)
                .ok_or(TournamentError::PlayerNotFound),
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
    pub fn get_player_by_id(&self, id: &PlayerId) -> Result<&Player, TournamentError> {
        self.player_reg.get_player(id)
    }

    /// Gets a copy of a player's registration data
    /// NOTE: This does not include their round data
    pub fn get_round_id(&self, ident: &RoundIdentifier) -> Result<RoundId, TournamentError> {
        match ident {
            RoundIdentifier::Id(id) => self
                .round_reg
                .validate_id(id)
                .then_some(*id)
                .ok_or(TournamentError::RoundLookup),
            RoundIdentifier::Number(num) => self.round_reg.get_round_id(num),
            RoundIdentifier::Table(num) => {
                self.round_reg.round_from_table_number(*num).map(|r| r.id)
            }
        }
    }

    /// Gets a copy of a round's data
    pub fn get_round(&self, ident: &RoundIdentifier) -> Result<&Round, TournamentError> {
        match ident {
            RoundIdentifier::Id(id) => self.round_reg.get_round(id),
            RoundIdentifier::Number(num) => self.round_reg.get_by_number(num),
            RoundIdentifier::Table(num) => self.round_reg.round_from_table_number(*num),
        }
    }

    /// Gets a copy of a round's data
    pub fn get_round_by_id(&self, id: &RoundId) -> Result<&Round, TournamentError> {
        self.round_reg.get_round(id)
    }

    /// Gets a vec of all active rounds
    pub fn get_active_rounds(&self) -> Vec<&Round> {
        self.round_reg
            .rounds
            .values()
            .filter(|r| r.is_active())
            .collect()
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
            .values()
            .filter_map(|r| r.players.contains(&id).then_some(r))
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
            .ok_or(TournamentError::DeckLookup)
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

    /// Removes players from the tournament that did not complete registration.
    /// This include players that did not submit enough decks (defined by `require_deck_reg` and
    /// `min_deck_count`) and that didn't check in (defined by `require_check_in`).
    pub(crate) fn prune_players(&mut self) -> OpResult {
        if !self.is_ongoing() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if self.settings.require_deck_reg {
            for p in self.player_reg.players.values_mut() {
                if p.decks.len() < self.settings.min_deck_count as usize {
                    p.update_status(PlayerStatus::Dropped);
                }
            }
        }
        if self.settings.require_check_in {
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
        if !self.is_ongoing() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let round = self.round_reg.get_mut_round(rnd)?;
        if let Some(new_extension) = round.extension.checked_add(ext) {
            round.extension = new_extension;
        } else {
            return Err(TournamentError::TimeOverflow);
        }
        Ok(OpData::Nothing)
    }

    /// Checks in a player for the tournament.
    pub(crate) fn check_in(&mut self, id: PlayerId) -> OpResult {
        if !self.is_ongoing() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if self.is_planned() {
            self.player_reg.check_in(id)?;
            Ok(OpData::Nothing)
        } else {
            Err(TournamentError::IncorrectStatus(self.status))
        }
    }

    /// Attempts to create the next set of rounds for the tournament
    pub(crate) fn pair(&mut self, salt: DateTime<Utc>, pairings: Pairings) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        self.pairing_sys.update(&pairings);
        let context = self.pairing_sys.get_context();
        Ok(OpData::Pair(
            self.round_reg.rounds_from_pairings(salt, pairings, context),
        ))
    }

    /// Attempts to create the next set of rounds for the tournament
    pub fn create_pairings(&self) -> Option<Pairings> {
        if !self.is_active() {
            return None;
        }
        let standings = self
            .scoring_sys
            .get_standings(&self.player_reg, &self.round_reg);
        self.pairing_sys
            .pair(&self.player_reg, &self.round_reg, standings)
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
            GeneralSetting(setting) => self.settings.update(setting),
            PairingSetting(setting) => self.pairing_sys.update_setting(setting),
            ScoringSetting(setting) => self.scoring_sys.update_setting(setting),
        }
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
        if !self.is_ongoing() {
            Err(TournamentError::IncorrectStatus(self.status))
        } else if !self.reg_open {
            Err(TournamentError::RegClosed)
        } else {
            let id = self.player_reg.register_player(account)?;
            Ok(OpData::RegisterPlayer(id))
        }
    }

    /// Records part of the result of a round
    pub(crate) fn record_result(&mut self, r_id: &RoundId, result: RoundResult) -> OpResult {
        if !self.is_active() {
            Err(TournamentError::IncorrectStatus(self.status))
        } else {
            self.round_reg.get_mut_round(r_id)?.record_result(result)?;
            Ok(OpData::Nothing)
        }
    }

    /// A player confirms the round record
    pub(crate) fn confirm_round(&mut self, r_id: RoundId, p_id: PlayerId) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let status = self.round_reg.get_mut_round(&r_id)?.confirm_round(p_id)?;
        Ok(OpData::ConfirmResult(r_id, status))
    }

    /// A judge or admin confirms the result of a match
    pub(crate) fn confirm_single_round(&mut self, id: &RoundId) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let round = self.round_reg.get_mut_round(id)?;
        match round.status {
            RoundStatus::Open if round.has_result() => {
                round.status = RoundStatus::Certified;
                Ok(OpData::Nothing)
            }
            RoundStatus::Open => Err(TournamentError::NoMatchResult),
            RoundStatus::Certified | RoundStatus::Dead => Err(TournamentError::RoundConfirmed),
        }
    }

    /// Confirms all active rounds in the tournament. If there is at least one active round without
    /// a result, this operations fails atomically.
    pub(crate) fn confirm_all_rounds(&mut self) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if self
            .round_reg
            .rounds
            .values()
            .filter(|r| r.is_active())
            .any(|r| !r.has_result())
        {
            return Err(TournamentError::NoMatchResult);
        }
        self.round_reg
            .rounds
            .values_mut()
            .filter(|r| r.is_active())
            .for_each(|r| {
                r.status = RoundStatus::Certified;
            });
        Ok(OpData::Nothing)
    }

    /// Dropps a player from the tournament
    pub(crate) fn drop_player(&mut self, id: PlayerId) -> OpResult {
        if self.is_dead() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        self.player_reg.drop_player(&id)?;
        for rnd in self.round_reg.get_player_active_rounds(&id) {
            rnd.drop_player(&id);
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
            rnd.drop_player(&id);
        }
        Ok(OpData::Nothing)
    }

    /// Adds a deck to a player's registration data
    pub(crate) fn player_add_deck(&mut self, id: PlayerId, name: String, deck: Deck) -> OpResult {
        if !self.is_ongoing() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if !self.reg_open {
            return Err(TournamentError::RegClosed);
        }
        self.add_deck(id, name, deck)
    }

    /// Removes a player's deck from their registration data
    pub(crate) fn remove_player_deck(&mut self, ident: &PlayerId, name: String) -> OpResult {
        if !self.is_ongoing() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let plyr = self.player_reg.get_mut_player(ident)?;
        plyr.remove_deck(name)?;
        Ok(OpData::Nothing)
    }

    /// Sets a player's gamer tag
    pub(crate) fn player_set_game_name(&mut self, ident: &PlayerId, name: String) -> OpResult {
        if !self.is_ongoing() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let plyr = self.player_reg.get_mut_player(ident)?;
        plyr.game_name = Some(name);
        Ok(OpData::Nothing)
    }

    /// Readies a player to play in their next round
    pub(crate) fn ready_player(&mut self, salt: DateTime<Utc>, ident: &PlayerId) -> OpResult {
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
        // FIXME: Pairings should be returned. Matches should not be created
        match should_pair {
            true => {
                let standings = self.get_standings();
                match self
                    .pairing_sys
                    .pair(&self.player_reg, &self.round_reg, standings)
                {
                    Some(pairings) => {
                        let context = self.pairing_sys.get_context();
                        let rounds = self.round_reg.rounds_from_pairings(salt, pairings, context);
                        Ok(OpData::Pair(rounds))
                    }
                    None => Ok(OpData::Nothing),
                }
            }
            false => Ok(OpData::Nothing),
        }
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
    pub(crate) fn give_bye(&mut self, salt: DateTime<Utc>, plyr: PlayerId) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        let context = self.pairing_sys.get_context();
        Ok(OpData::GiveBye(
            self.round_reg.give_bye(salt, plyr, context),
        ))
    }

    /// Creates a new round from a list of players
    pub fn create_round(&mut self, salt: DateTime<Utc>, plyrs: Vec<PlayerId>) -> OpResult {
        if !self.is_active() {
            return Err(TournamentError::IncorrectStatus(self.status));
        }
        if plyrs.len() == self.pairing_sys.common.match_size as usize
            && plyrs.iter().all(|p| self.player_reg.is_registered(p))
        {
            let context = self.pairing_sys.get_context();
            Ok(OpData::CreateRound(
                self.round_reg.create_round(salt, plyrs, context),
            ))
        } else {
            Err(TournamentError::PlayerNotFound)
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
        if !self.is_ongoing() {
            Err(TournamentError::IncorrectStatus(self.status))
        } else {
            Ok(OpData::RegisterPlayer(
                self.player_reg.register_player(account)?,
            ))
        }
    }

    fn register_guest(&mut self, salt: DateTime<Utc>, name: String) -> OpResult {
        if !self.is_ongoing() {
            Err(TournamentError::IncorrectStatus(self.status))
        } else {
            Ok(OpData::RegisterPlayer(
                self.player_reg.add_guest(salt, name)?,
            ))
        }
    }

    fn reregister_guest(&mut self, name: String) -> OpResult {
        if !self.is_ongoing() {
            Err(TournamentError::IncorrectStatus(self.status))
        } else {
            self.player_reg.reregister_guest(name)?;
            Ok(OpData::Nothing)
        }
    }

    fn register_judge(&mut self, account: SquireAccount) -> OpResult {
        if !self.is_ongoing() {
            Err(TournamentError::IncorrectStatus(self.status))
        } else {
            let judge = Judge::new(account);
            self.judges.insert(judge.id, judge.clone());
            Ok(OpData::RegisterJudge(judge))
        }
    }

    fn register_admin(&mut self, account: SquireAccount) -> OpResult {
        if !self.is_ongoing() {
            Err(TournamentError::IncorrectStatus(self.status))
        } else {
            let admin = Admin::new(account);
            self.admins.insert(admin.id, admin.clone());
            Ok(OpData::RegisterAdmin(admin))
        }
    }

    fn admin_add_deck(&mut self, id: PlayerId, name: String, deck: Deck) -> OpResult {
        if self.is_ongoing() {
            self.add_deck(id, name, deck)
        } else {
            Err(TournamentError::IncorrectStatus(self.status))
        }
    }

    // Common code between admin and player versions.
    // NOTE: Does *not* contain checks for tournament/player validity. Those are handled by the
    // admin and player versions
    fn add_deck(&mut self, id: PlayerId, name: String, deck: Deck) -> OpResult {
        let plyr = self.player_reg.get_mut_player(&id)?;
        if !plyr.decks.contains_key(&name)
            && plyr.decks.len() >= self.settings.max_deck_count as usize
        {
            return Err(TournamentError::MaxDecksReached);
        }
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

    fn admin_ready_player(&mut self, salt: DateTime<Utc>, id: PlayerId) -> OpResult {
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
        // FIXME: Pairings should be returned. Matches should not be created
        match should_pair {
            true => {
                let standings = self.get_standings();
                match self
                    .pairing_sys
                    .pair(&self.player_reg, &self.round_reg, standings)
                {
                    Some(pairings) => {
                        let context = self.pairing_sys.get_context();
                        let rounds = self.round_reg.rounds_from_pairings(salt, pairings, context);
                        Ok(OpData::Pair(rounds))
                    }
                    None => Ok(OpData::Nothing),
                }
            }
            false => Ok(OpData::Nothing),
        }
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
        if self.settings.require_deck_reg {
            digest.0 = self
                .player_reg
                .players
                .values()
                .filter(|p| p.decks.len() < self.settings.min_deck_count as usize)
                .count();
        }
        if self.settings.require_check_in {
            digest.1 = self.player_reg.len() - self.player_reg.check_ins.len();
        }
        digest
    }

    /// Returns the complete set of all current settings in the tournament
    pub fn settings(&self) -> TournamentSettingsTree {
        TournamentSettingsTree {
            general: self.settings.clone(),
            pairing: self.pairing_sys.settings(),
            scoring: self.scoring_sys.settings(),
        }
    }

    /// Generates a round slip in HTML
    pub fn round_slips_html(&self, css: &str) -> String {
        let mut ret: String = "<!DOCTPYE HTML>".to_string();

        let _ = write!(ret, "<html lang=\"en\">");
        let _ = write!(ret, "<head><style>");
        let _ = write!(ret, "{}", &css);
        let _ = write!(ret, "</style></head>");
        let _ = write!(ret, "<body>");

        for r in self.round_reg.rounds.values().filter(|r| r.is_active()) {
            let _ = write!(ret, "<div class='card' style='border-style: solid;'>");
            let _ = write!(ret, "<div class='title' style='display: flex; align-items: center; width: 100%; flex-direction: column;'>");
            let _ = write!(ret, "{}", &html_escape::encode_text(&self.name.to_string()));
            let _ = write!(ret, "</div>");
            let _ = write!(ret, "<table style='width: 100%;'>");
            let _ = write!(ret, "<tr><tr><td>Round # ");
            let _ = write!(ret, "{}", r.match_number);
            let _ = write!(ret, "</td>");

            if self.settings.use_table_number {
                let _ = write!(ret, "<td>Table #");
                let _ = write!(ret, "{}", &r.table_number.to_string());
                let _ = write!(ret, "</td><tr>");
            }

            for (i, pid) in r.players.iter().enumerate() {
                if i % 2 == 0 {
                    if i != 0 {
                        let _ = write!(ret, "</tr>");
                    }
                    let _ = write!(ret, "<tr>");
                }
                let _ = write!(ret, "<td>");
                let _ = write!(
                    ret,
                    "{}",
                    &html_escape::encode_text(
                        &self
                            .player_reg
                            .get_player(pid)
                            .expect("Round's playes should be within the tournament.")
                            .all_names(),
                    )
                );
                let _ = write!(ret, "</td>");
            }
            let _ = write!(ret, "</tr></table></div>");
        }
        let _ = write!(ret, "</body>");

        ret
    }
}

impl TournamentSeed {
    /// Creates a new tournament seed
    pub fn new(name: String, preset: TournamentPreset, format: String) -> Self {
        Self {
            name,
            preset,
            format,
        }
    }
}

impl TryFrom<TournamentSeed> for Tournament {
    type Error = TournamentError;

    fn try_from(seed: TournamentSeed) -> Result<Self, Self::Error> {
        let TournamentSeed {
            name,
            preset,
            format,
        } = seed;
        Tournament::from_preset(name, preset, format)
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::Utc;
    use uuid::Uuid;

    use super::{Tournament, TournamentPreset};
    use crate::{
        accounts::{SharingPermissions, SquireAccount},
        admin::Admin,
        operations::{AdminOp, PlayerOp, TournOp},
        rounds::RoundResult,
        tournament::TournamentSeed,
    };

    fn spoof_account() -> SquireAccount {
        let id = Uuid::new_v4().into();
        SquireAccount {
            id,
            user_name: id.to_string(),
            display_name: id.to_string(),
            gamer_tags: HashMap::new(),
            permissions: SharingPermissions::Everything,
        }
    }

    #[test]
    fn players_in_paired_rounds() {
        let mut tourn =
            Tournament::from_preset("Test".into(), TournamentPreset::Swiss, "Test".into()).unwrap();
        assert_eq!(tourn.pairing_sys.common.match_size, 2);
        let acc = spoof_account();
        let admin = Admin::new(acc);
        tourn.admins.insert(admin.id, admin.clone());
        let acc = spoof_account();
        tourn
            .apply_op(Utc::now(), TournOp::RegisterPlayer(acc))
            .unwrap()
            .assume_register_player();
        let acc = spoof_account();
        tourn
            .apply_op(Utc::now(), TournOp::RegisterPlayer(acc))
            .unwrap()
            .assume_register_player();
        tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin.id, AdminOp::Start))
            .unwrap()
            .assume_nothing();
        let pairings = tourn.create_pairings().unwrap();
        let r_ids = tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(admin.id, AdminOp::PairRound(pairings)),
            )
            .unwrap()
            .assume_pair();
        assert_eq!(r_ids.len(), 1);
        let rnd = tourn.get_round_by_id(&r_ids[0]).unwrap();
        assert_eq!(rnd.players.len(), 2);
    }

    #[test]
    fn confirm_all_rounds_test() {
        let mut tourn =
            Tournament::from_preset("Test".into(), TournamentPreset::Swiss, "Test".into()).unwrap();
        assert_eq!(tourn.pairing_sys.common.match_size, 2);
        let acc = spoof_account();
        let admin = Admin::new(acc);
        tourn.admins.insert(admin.id, admin.clone());
        let mut plyrs = Vec::with_capacity(4);
        for _ in 0..4 {
            let acc = spoof_account();
            let id = tourn
                .apply_op(Utc::now(), TournOp::RegisterPlayer(acc))
                .unwrap()
                .assume_register_player();
            plyrs.push(id);
        }
        tourn
            .apply_op(Utc::now(), TournOp::AdminOp(admin.id, AdminOp::Start))
            .unwrap()
            .assume_nothing();
        // Pair the first round
        let pairings = tourn.create_pairings().unwrap();
        let rnds = tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(admin.id, AdminOp::PairRound(pairings)),
            )
            .unwrap()
            .assume_pair();
        assert_eq!(rnds.len(), 2);
        let r_id = tourn
            .round_reg
            .get_player_active_round(&plyrs[0])
            .unwrap()
            .id;
        tourn
            .apply_op(
                Utc::now(),
                TournOp::PlayerOp(
                    plyrs[0],
                    PlayerOp::RecordResult(r_id, RoundResult::Wins(plyrs[0], 1)),
                ),
            )
            .unwrap()
            .assume_nothing();
        assert!(tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(admin.id, AdminOp::ConfirmAllRounds)
            )
            .is_err());
        for p in plyrs {
            let r_id = tourn.round_reg.get_player_active_round(&p).unwrap().id;
            tourn
                .apply_op(
                    Utc::now(),
                    TournOp::PlayerOp(p, PlayerOp::RecordResult(r_id, RoundResult::Wins(p, 1))),
                )
                .unwrap()
                .assume_nothing();
        }
        tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(admin.id, AdminOp::ConfirmAllRounds),
            )
            .unwrap()
            .assume_nothing();
        // Pair the second round
        let pairings = tourn.create_pairings().unwrap();
        tourn
            .apply_op(
                Utc::now(),
                TournOp::AdminOp(admin.id, AdminOp::PairRound(pairings)),
            )
            .unwrap()
            .assume_pair();
    }

    #[test]
    fn valid_tournament_names() {
        fn seed(name: &str) -> TournamentSeed {
            TournamentSeed {
                name: name.to_string(),
                preset: TournamentPreset::Fluid,
                format: "Test".to_string(),
            }
        }

        assert!(Tournament::try_from(seed("")).is_err());
        assert!(Tournament::try_from(seed("😄")).is_err());
        assert!(Tournament::try_from(seed("abc")).is_ok());
        assert!(Tournament::try_from(seed("_!(:)#@")).is_ok());
        assert!(Tournament::try_from(seed("Magic: the Gathering")).is_ok());
        assert!(Tournament::try_from(seed("Prophecy: the Body")).is_ok());
    }
}

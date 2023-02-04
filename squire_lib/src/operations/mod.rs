#![allow(dead_code, unused_variables)]
use std::borrow::Cow;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    accounts::SquireAccount,
    admin::{Admin, Judge, TournOfficialId},
    error::TournamentError,
    identifiers::{id_from_item, AdminId, JudgeId, OpId, PlayerId},
    rounds::{RoundId, RoundStatus},
    settings::TournamentSetting,
    tournament::{TournamentPreset, TournamentStatus},
};

mod admin_ops;
mod judge_ops;
mod op_log;
mod op_sync;
mod player_ops;

pub use admin_ops::AdminOp;
pub use judge_ops::JudgeOp;
pub use op_log::*;
pub use op_sync::*;
pub use player_ops::PlayerOp;

#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
/// This enum captures all ways in which a tournament can mutate.
pub enum TournOp {
    /// Operation to mark the creation of a tournament
    Create(SquireAccount, String, TournamentPreset, String),
    /// Operation for a player register themself for a tournament
    RegisterPlayer(SquireAccount),
    /// Opertions that a player can perform, after registering
    PlayerOp(PlayerId, PlayerOp),
    /// Opertions that a judges or admins can perform
    JudgeOp(TournOfficialId, JudgeOp),
    /// Opertions that a only admin can perform
    AdminOp(AdminId, AdminOp),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// An enum that encodes all possible data after successfully applying a tournament operation
pub enum OpData {
    /// There is no data to be returned
    Nothing,
    /// A player was registerd and this is their id
    RegisterPlayer(PlayerId),
    /// A player was registerd and this is their id
    RegisterJudge(Judge),
    /// A player was registerd and this is their id
    RegisterAdmin(Admin),
    /// A round result was confirmed and this is the current status of that round
    ConfirmResult(RoundId, RoundStatus),
    /// A player was given a bye and this is the id of that round
    GiveBye(RoundId),
    /// A round was manually created and this is that round's id
    CreateRound(RoundId),
    /// The next set of rounds was paired and these are those round's ids
    Pair(Vec<RoundId>),
}

/// A shorthand for the outcome of attempting to apply an operation to a tournament
pub type OpResult = Result<OpData, TournamentError>;

impl OpData {
    /// Calculates if the data is nothing
    pub fn is_nothing(&self) -> bool {
        matches!(self, Self::Nothing)
    }

    /// Assumes contained data is `Nothing`
    ///
    /// PANICS: If the data is anything else, this method panics.
    pub fn assume_nothing(self) {
        match self {
            Self::Nothing => (),
            _ => panic!("Assumed OpData nothing failed"),
        }
    }

    /// Assumes contained data is from `RegisterPlayer` and returns that id, analogous to `unwrap`.
    ///
    /// PANICS: If the data is anything else, this method panics.
    pub fn assume_register_player(self) -> PlayerId {
        match self {
            Self::RegisterPlayer(id) => id,
            _ => panic!("Assumed OpData was register player failed"),
        }
    }

    /// Assumes contained data is from `RegisterJudge` and returns that id, analogous to `unwrap`.
    ///
    /// PANICS: If the data is anything else, this method panics.
    pub fn assume_register_judge(self) -> Judge {
        match self {
            Self::RegisterJudge(judge) => judge,
            _ => panic!("Assumed OpData was register judge failed"),
        }
    }

    /// Assumes contained data is from `RegisterAdmin` and returns that id, analogous to `unwrap`.
    ///
    /// PANICS: If the data is anything else, this method panics.
    pub fn assume_register_admin(self) -> Admin {
        match self {
            Self::RegisterAdmin(admin) => admin,
            _ => panic!("Assumed OpData was register admin failed"),
        }
    }

    /// Assumes contained data is from `ConfirmResult` and returns that id, analogous to `unwrap`.
    ///
    /// PANICS: If the data is anything else, this method panics.
    pub fn assume_confirm_result(self) -> (RoundId, RoundStatus) {
        match self {
            Self::ConfirmResult(r_id, status) => (r_id, status),
            _ => panic!("Assumed OpData was confirm result failed"),
        }
    }

    /// Assumes contained data is from `GiveBye` and returns that id, analogous to `unwrap`.
    ///
    /// PANICS: If the data is anything else, this method panics.
    pub fn assume_give_bye(self) -> RoundId {
        match self {
            Self::GiveBye(id) => id,
            _ => panic!("Assumed OpData was give bye failed"),
        }
    }

    /// Assumes contained data is from `CreateRound` and returns that id, analogous to `unwrap`.
    ///
    /// PANICS: If the data is anything else, this method panics.
    pub fn assume_create_round(self) -> RoundId {
        match self {
            Self::CreateRound(id) => id,
            _ => panic!("Assumed OpData was create round failed"),
        }
    }

    /// Assumes contained data is from `Pair` and returns that id, analogous to `unwrap`.
    ///
    /// PANICS: If the data is anything else, this method panics.
    pub fn assume_pair(self) -> Vec<RoundId> {
        match self {
            Self::Pair(ids) => ids,
            _ => panic!("Assumed OpData was pair round failed"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// An full operation used by the tournament manager to help track metadata for client-server
/// syncing
pub struct FullOp {
    pub(crate) op: TournOp,
    pub(crate) salt: DateTime<Utc>,
    pub(crate) id: OpId,
    pub(crate) active: bool,
}

/// An enum that captures the ways in which two `FullOp`s can differ. This is a vital part in the
/// tournament syncing process.
#[derive(Debug, Clone, Copy)]
pub enum OpDiff {
    /// The two operations are completely equal
    Equal,
    /// The two operations
    Inactive,
    /// The two operations are completely equal
    Time,
    /// The two operations are completely equal
    Different,
}

impl FullOp {
    /// Creates a new FullOp from an existing TournOp
    pub fn new(op: TournOp) -> Self {
        let salt = Utc::now();
        let id = id_from_item(salt, &op);
        Self {
            op,
            id,
            salt,
            active: true,
        }
    }
    
    pub(crate) fn get_update(&self) -> OpUpdate {
        self.op.get_update(self.salt)
    }
    
    pub(crate) fn swap_player_ids(&mut self, old: PlayerId, new: PlayerId) {
        self.op.swap_player_ids(old, new)
    }
    
    pub(crate) fn swap_round_ids(&mut self, old: RoundId, new: RoundId) {
        self.op.swap_round_ids(old, new)
    }
    
    /// Calculate the kind of difference (if any) there is between two operations
    pub fn diff(&self, other: &Self) -> OpDiff {
        if self.op != other.op {
            OpDiff::Different
        } else if self.active != other.active {
            OpDiff::Inactive
        } else if self.salt != other.salt {
            OpDiff::Time
        } else {
            OpDiff::Equal
        }
    }

    /// Determines if the given operation affects this operation
    pub fn blocks(&self, other: &Self) -> bool {
        self.op.blocks(&other.op)
    }
}

/// Encodes what an operations relates on or affects in a tournaments
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub(crate) enum OpEffects {
    Player(OpPlayerEffects),
    Round(OpRoundEffects),
    Settings(TournamentSetting),
    Status(TournamentStatus),
    Admin(OpAdminEffects),
    Everything,
}

/// Encodes the ways in which an operation can affect players
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub(crate) enum OpPlayerEffects {
    Single(PlayerId, PlayerEffectComponent),
    SingleActive(PlayerId, PlayerEffectComponent),
    All(PlayerEffectComponent),
    AllActive(PlayerEffectComponent),
}

/// Encodes the what an operation affects about a player
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub(crate) enum PlayerEffectComponent {
    Nothing,
    Deck(String),
    Status,
    CheckIn,
}

/// Encodes the ways in which an operation can affect rounds
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub(crate) enum OpRoundEffects {
    Single(RoundId, RoundEffectComponent),
    SingleActive(RoundId, RoundEffectComponent),
    All(RoundEffectComponent),
    AllActive(RoundEffectComponent),
}

/// Encodes the what an operation affects about a player
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub(crate) enum RoundEffectComponent {
    Nothing,
    Result(Option<PlayerId>),
    Confirmation,
    Status,
}

/// Encodes the ways in which an operation can affect rounds
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub(crate) enum OpAdminEffects {
    Judge(JudgeId),
    Admin(AdminId),
}

pub(crate) struct OpGroup {
    pub(crate) effects: Cow<'static, [OpEffects]>,
}

#[derive(Debug, Clone)]
/// Encapsules the ways that an operation might need to be updated during the sync process
pub(crate) enum OpUpdate {
    None,
    PlayerId(PlayerId),
    RoundId(Vec<RoundId>),
}

impl OpUpdate {
    pub(crate) fn assume_player_id(self) -> PlayerId {
        match self {
            OpUpdate::None => panic!("OpUpdate assumed to be PlayerId but was None"),
            OpUpdate::PlayerId(id) => id,
            OpUpdate::RoundId(_) => panic!("OpUpdate assumed to be PlayerId but was RoundId"),
        }
    }
    
    pub(crate) fn assume_round_id(self) -> Vec<RoundId> {
        match self {
            OpUpdate::None => panic!("OpUpdate assumed to be RoundId but was None"),
            OpUpdate::PlayerId(_) => panic!("OpUpdate assumed to be RoundId but was PlayerId"),
            OpUpdate::RoundId(id) => id,
        }
    }
}

impl TournOp {
    /// Determines if this operation blocks a given operation
    pub fn blocks(&self, other: &Self) -> bool {
        self.affects().blocks(other.requires())
    }
    
    pub(crate) fn get_update(&self, salt: DateTime<Utc>) -> OpUpdate {
        match self {
            TournOp::Create(_, _, _, _) => OpUpdate::None,
            TournOp::RegisterPlayer(_) => OpUpdate::None,
            TournOp::PlayerOp(_, p_op) => p_op.get_update(salt),
            TournOp::JudgeOp(_, j_op) => j_op.get_update(salt),
            TournOp::AdminOp(_, a_op) => a_op.get_update(salt),
        }
    }
    
    pub(crate) fn swap_player_ids(&mut self, old: PlayerId, new: PlayerId) {
        match self {
            TournOp::Create(_, _, _, _) => { },
            TournOp::RegisterPlayer(_) => { },
            TournOp::PlayerOp(p_id, _) => {
                if *p_id == old {
                    *p_id = new;
                }
            }
            TournOp::JudgeOp(_, j_op) => j_op.swap_player_ids(old, new),
            TournOp::AdminOp(_, a_op) => a_op.swap_player_ids(old, new),
        }
    }
    
    pub(crate) fn swap_round_ids(&mut self, old: RoundId, new: RoundId) {
        match self {
            TournOp::Create(_, _, _, _) => { },
            TournOp::RegisterPlayer(_) => { },
            TournOp::PlayerOp(_, p_op) => p_op.swap_round_ids(old, new),
            TournOp::JudgeOp(_, j_op) => j_op.swap_round_ids(old, new),
            TournOp::AdminOp(_, a_op) => a_op.swap_round_ids(old, new),
        }
    }

    fn affects(&self) -> OpGroup {
        match self {
            TournOp::Create(..) => OpGroup {
                effects: Cow::Borrowed(&[]),
            },
            TournOp::RegisterPlayer(..) => OpGroup {
                effects: Cow::Borrowed(&[]),
            },
            TournOp::PlayerOp(p_id, op) => op.affects(*p_id),
            TournOp::JudgeOp(_, op) => op.affects(),
            TournOp::AdminOp(a_id, op) => op.affects(*a_id),
        }
    }

    fn requires(&self) -> OpGroup {
        match self {
            TournOp::Create(..) => OpGroup {
                effects: Cow::Borrowed(&[]),
            },
            TournOp::RegisterPlayer(account) => OpGroup {
                effects: Cow::Borrowed(&[]),
            },
            TournOp::PlayerOp(p_id, op) => op.requires(*p_id),
            TournOp::JudgeOp(to_id, op) => op.requires(*to_id),
            TournOp::AdminOp(a_id, op) => op.requires(*a_id),
        }
    }
}

impl OpGroup {
    fn blocks(&self, _other: Self) -> bool {
        todo!()
    }
}

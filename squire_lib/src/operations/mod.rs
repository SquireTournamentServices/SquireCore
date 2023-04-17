
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    accounts::SquireAccount,
    admin::{Admin, Judge, TournOfficialId},
    error::TournamentError,
    identifiers::{AdminId, PlayerId},
    rounds::{RoundId, RoundStatus},
};

mod admin_ops;
mod judge_ops;
mod player_ops;

pub use admin_ops::AdminOp;
pub use judge_ops::JudgeOp;
pub use player_ops::PlayerOp;

#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
/// This enum captures all ways in which a tournament can mutate.
pub enum TournOp {
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

#[derive(Debug, Clone)]
/// Encapsules the ways that an operation might need to be updated during the sync process
pub enum OpUpdate {
    /// This operation has no update
    None,
    /// This operation has a player id that can be updated
    PlayerId(PlayerId),
    /// This operation has one or more round ids that can be updated
    RoundId(Vec<RoundId>),
}

impl OpUpdate {
    /// Unwraps the update. Returns the player id if it exists and panics otherwise.
    pub fn assume_player_id(self) -> PlayerId {
        match self {
            OpUpdate::None => panic!("OpUpdate assumed to be PlayerId but was None"),
            OpUpdate::PlayerId(id) => id,
            OpUpdate::RoundId(_) => panic!("OpUpdate assumed to be PlayerId but was RoundId"),
        }
    }

    /// Unwraps the update. Returns the round id(s) if present and panics otherwise.
    pub fn assume_round_id(self) -> Vec<RoundId> {
        match self {
            OpUpdate::None => panic!("OpUpdate assumed to be RoundId but was None"),
            OpUpdate::PlayerId(_) => panic!("OpUpdate assumed to be RoundId but was PlayerId"),
            OpUpdate::RoundId(id) => id,
        }
    }
}

impl TournOp {
    /// Returns the update that the operation can have
    pub fn get_update(&self, salt: DateTime<Utc>) -> OpUpdate {
        match self {
            TournOp::RegisterPlayer(_) => OpUpdate::None,
            TournOp::PlayerOp(_, p_op) => p_op.get_update(salt),
            TournOp::JudgeOp(_, j_op) => j_op.get_update(salt),
            TournOp::AdminOp(_, a_op) => a_op.get_update(salt),
        }
    }

    /// Replaces an old player id with a new player id in the operation
    pub fn swap_player_ids(&mut self, old: PlayerId, new: PlayerId) {
        match self {
            TournOp::RegisterPlayer(_) => {}
            TournOp::PlayerOp(p_id, _) => {
                if *p_id == old {
                    *p_id = new;
                }
            }
            TournOp::JudgeOp(_, j_op) => j_op.swap_player_ids(old, new),
            TournOp::AdminOp(_, a_op) => a_op.swap_player_ids(old, new),
        }
    }

    /// Replaces an old round id with a new round id in the operation
    pub fn swap_round_ids(&mut self, old: RoundId, new: RoundId) {
        match self {
            TournOp::RegisterPlayer(_) => {}
            TournOp::PlayerOp(_, p_op) => p_op.swap_round_ids(old, new),
            TournOp::JudgeOp(_, j_op) => j_op.swap_round_ids(old, new),
            TournOp::AdminOp(_, a_op) => a_op.swap_round_ids(old, new),
        }
    }
}

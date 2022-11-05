use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    accounts::SquireAccount,
    admin::{Admin, Judge, TournOfficialId},
    error::TournamentError,
    identifiers::{AdminId, OpId, PlayerId},
    rounds::{RoundId, RoundStatus},
    tournament::TournamentPreset, pairings::Pairings,
};

mod admin_ops;
mod judge_ops;
mod player_ops;

pub use admin_ops::AdminOp;
pub use judge_ops::JudgeOp;
pub use player_ops::PlayerOp;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
    /// The pairings for the next set of rounds
    CreatePairings(Pairings),
    /// The next set of rounds was paired and these are those round's ids
    Pair(Vec<RoundId>),
}

/// A shorthand for the outcome of attempting to apply an operation to a tournament
pub type OpResult = Result<OpData, TournamentError>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// An full operation used by the tournament manager to help track metadata for client-server
/// syncing
pub struct FullOp {
    pub(crate) op: TournOp,
    pub(crate) salt: DateTime<Utc>,
    pub(crate) id: OpId,
    pub(crate) active: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// An ordered list of all operations applied to a tournament
pub struct OpLog {
    pub(crate) ops: Vec<FullOp>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// An ordered list of some of the operations applied to a tournament
pub struct OpSlice {
    pub(crate) ops: Vec<FullOp>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A struct to help resolve syncing op logs
pub struct OpSync {
    pub(crate) ops: OpSlice,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// An enum to help track the progress of the syncing of two op logs
pub enum SyncStatus {
    /// There was an error when attempting to initially sync
    SyncError(SyncError),
    /// There are discrepancies in between the two logs that are being synced
    InProgress(Blockage),
    /// The logs have been successfully syncs
    Completed(OpSync),
}

/// An enum to that captures the error that might occur when sync op logs.
/// `UnknownOperation` encodes that first operation in an OpSlice is unknown
/// `RollbackFound` encode that a rollback has occured remotely but not locally and returns an
/// OpSlice that contains everything since that rollback. When recieved, this new log should
/// overwrite the local log
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SyncError {
    /// One of the log was empty
    EmptySync,
    /// The starting operation of the slice in unknown to the other log
    UnknownOperation(FullOp),
    /// One of the logs contains a rollback that the other doesn't have
    RollbackFound(OpSlice),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// An enum that encodes that errors that can occur during a rollback
pub enum RollbackError {
    /// The rollback slice has an unknown starting point
    SliceError(SyncError),
    /// The log that doesn't contain the rollback contains operations that the rolled back log
    /// doesn't contain
    OutOfSync(OpSync),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A struct to help resolve blockages
pub struct Blockage {
    pub(crate) known: OpSlice,
    pub(crate) agreed: OpSlice,
    pub(crate) other: OpSlice,
    pub(crate) problem: (FullOp, FullOp),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A struct used to communicate a rollback
pub struct Rollback {
    pub(crate) ops: OpSlice,
}

impl Blockage {
    /// Returns the problematic pair of operations.
    pub fn problem(&self) -> (FullOp, FullOp) {
        self.problem.clone()
    }

    /// Resolves the current problem by keeping the given solution and deleting the other, consuming self.
    ///
    /// The `Ok` varient is returned if the given operation is one of the problematic operations.
    /// It contains the rectified logs.
    ///
    /// The `Err` varient is returned if the given operation isn't one of the problematic operations, containing `self`.
    pub fn pick_and_continue(mut self, op: FullOp) -> SyncStatus {
        if op == self.problem.0 {
            self.agreed.add_op(self.problem.0.clone());
        } else if op == self.problem.1 {
            self.agreed.add_op(self.problem.1.clone());
        } else {
            return SyncStatus::InProgress(self);
        }
        self.attempt_resolution()
    }

    /// Resolves the current problem by ordering the problematic solutions, consuming self.
    ///
    /// The `Ok` varient is returned if the given operation is one of the problematic operations.
    /// It contains the rectified logs.
    ///
    /// The `Err` varient is returned if the given operation isn't one of the problematic operations, containing `self`.
    pub fn order_and_continue(mut self, first: FullOp) -> SyncStatus {
        if first == self.problem.0 {
            self.agreed.add_op(self.problem.0.clone());
            self.agreed.add_op(self.problem.1.clone());
        } else if first == self.problem.1 {
            self.agreed.add_op(self.problem.1.clone());
            self.agreed.add_op(self.problem.0.clone());
        } else {
            return SyncStatus::InProgress(self);
        }
        self.attempt_resolution()
    }

    /// Resolves the current problem by applying exactly one operation and putting the other back
    /// in its slice, consuming self.
    pub fn push_and_continue(mut self, apply: FullOp) -> SyncStatus {
        if apply == self.problem.0 {
            self.agreed.add_op(self.problem.0.clone());
            self.other.ops.insert(0, self.problem.1.clone());
        } else if apply == self.problem.1 {
            self.agreed.add_op(self.problem.1.clone());
            self.known.ops.insert(0, self.problem.0.clone());
        } else {
            return SyncStatus::InProgress(self);
        }
        self.attempt_resolution()
    }

    fn attempt_resolution(mut self) -> SyncStatus {
        match self.known.merge(self.other) {
            SyncStatus::Completed(sync) => {
                self.agreed.ops.extend(sync.ops.ops.into_iter());
                SyncStatus::Completed(OpSync { ops: self.agreed })
            }
            SyncStatus::InProgress(mut block) => {
                self.agreed.ops.extend(block.agreed.ops.into_iter());
                block.agreed = self.agreed;
                SyncStatus::InProgress(block)
            }
            SyncStatus::SyncError(e) => match e {
                SyncError::RollbackFound(roll) => {
                    SyncStatus::SyncError(SyncError::RollbackFound(roll))
                }
                SyncError::UnknownOperation(_) => {
                    unreachable!("There should be no unknown starting operations during the resolution of a blockage.");
                }
                SyncError::EmptySync => {
                    unreachable!(
                        "There should be no empty syncs during the resolution of a blockage"
                    );
                }
            },
        }
    }
}

impl OpLog {
    /// Creates a new log
    pub fn new(op: FullOp) -> Self {
        let mut ops = Vec::new();
        ops.push(op);
        OpLog { ops }
    }

    /// Adds an operation to the end of the OpLog
    pub fn add_op(&mut self, op: FullOp) {
        self.ops.push(op);
    }

    /// Creates a slice of this log starting at the given index. `None` is returned if `index` is
    /// out of bounds.
    pub(crate) fn get_slice(&self, id: OpId) -> Option<OpSlice> {
        self.get_slice_extra(id, 0)
    }

    /// Creates a slice of this log starting at the given index. `None` is returned if `index` is
    /// out of bounds.
    pub(crate) fn get_slice_extra(&self, id: OpId, mut extra: usize) -> Option<OpSlice> {
        let mut end = false;
        let mut ops: Vec<FullOp> = Vec::new();
        for i_op in self.ops.iter().rev().cloned() {
            if end && extra == 0 {
                break;
            }
            if end {
                extra -= 1;
            }
            end |= i_op.id == id;
            ops.push(i_op);
        }
        if !end && extra != 0 {
            return None;
        }
        Some(OpSlice {
            ops: ops.into_iter().rev().collect(),
        })
    }

    pub(crate) fn slice_from_slice(&self, ops: &OpSlice) -> Result<OpSlice, SyncError> {
        let op = match ops.start_op() {
            Some(op) => op,
            None => {
                return Err(SyncError::EmptySync);
            }
        };
        match self.get_slice(op.id) {
            Some(slice) => {
                if slice.start_op().unwrap() != op {
                    return Err(SyncError::RollbackFound(slice));
                }
                Ok(slice)
            }
            None => Err(SyncError::UnknownOperation(op)),
        }
    }

    /// Removes all elements in the log starting at the first index of the given slice. All
    /// operations in the slice are then appended to the end of the log.
    pub fn overwrite(&mut self, ops: OpSlice) -> Result<(), SyncError> {
        let slice = self.slice_from_slice(&ops)?;
        let id = slice.start_id().unwrap();
        let index = self.ops.iter().position(|o| o.id == id).unwrap();
        self.ops.truncate(index);
        self.ops.extend(ops.ops.into_iter());
        Ok(())
    }

    /// Creates a slice of the current log by starting at the end and moving back. All operations
    /// that cause the closure to return `true` will be dropped and `false` will be kept. An
    /// operation causes `None` to be returned will end the iteration, will not be in the slice,
    /// but kept in the log.
    pub fn create_rollback(&self, id: OpId) -> Option<Rollback> {
        let mut ops = self.get_slice_extra(id, 1)?;
        ops.ops.iter_mut().skip(1).for_each(|op| op.active = false);
        Some(Rollback { ops })
    }

    /// Applies a rollback to this log.
    /// Err is returned if there is a different in between the length of the given slice and the
    /// corresponding slice of this log, and this log is not changed.
    /// Otherwise, the rollback is simply applied.
    ///
    /// NOTE: An OpSync is returned as the error data because the sender needs to have an
    /// up-to-date history before sendings a rollback.
    pub fn apply_rollback(&mut self, rollback: Rollback) -> Result<(), RollbackError> {
        let slice = self
            .slice_from_slice(&rollback.ops)
            .map_err(RollbackError::SliceError)?;
        if slice.ops.len() > rollback.ops.ops.len() {
            return Err(RollbackError::OutOfSync(OpSync { ops: slice }));
        }
        let mut r_op = rollback.ops.ops.iter();
        for i_op in slice.ops.iter() {
            let mut broke = false;
            while let Some(r) = r_op.next() {
                // If the id is unknown, the operation is unknow... so we continue.
                // Unknown, inactive ops ok to keep around. They can't affect anything
                if i_op.id == r.id {
                    broke = true;
                    break;
                }
            }
            if !broke {
                return Err(RollbackError::OutOfSync(OpSync { ops: slice }));
            }
        }
        // This should never return an Err
        self.overwrite(rollback.ops)
            .map_err(RollbackError::SliceError)
    }

    /// Attempts to sync the local log with a remote log.
    /// Returns Err if the starting op id of the given log can't be found in this log.
    /// Otherwise, Ok is returned and contains a SyncStatus
    pub fn sync(&mut self, other: OpSync) -> SyncStatus {
        let slice = match self.slice_from_slice(&other.ops) {
            Ok(s) => s,
            Err(e) => {
                return SyncStatus::SyncError(e);
            }
        };
        slice.merge(other.ops)
    }
}

impl OpSlice {
    /// Creates a new slice
    pub fn new() -> Self {
        OpSlice { ops: Vec::new() }
    }

    /// Adds an operation to the end of the OpSlice
    pub fn add_op(&mut self, op: FullOp) {
        self.ops.push(op);
    }

    /// Returns the index of the first stored operation.
    pub fn start_op(&self) -> Option<FullOp> {
        self.ops.first().cloned()
    }

    /// Returns the index of the first stored operation.
    pub fn start_id(&self) -> Option<OpId> {
        self.ops.first().map(|o| o.id)
    }

    /// Takes the slice and strips all inactive operations. This is only needed in the unlikely
    /// scenerio where a client rollbacks without communicating with the server and then tries to
    /// sync with the server.
    pub fn squash(self) -> Self {
        Self {
            ops: self.ops.into_iter().filter(|o| o.active).collect(),
        }
    }

    /// Takes another op slice and attempts to merge it with this slice.
    ///
    /// If there are no blockages, the `Ok` varient is returned containing the rectified log and
    /// this log is updated.
    ///
    /// If there is a blockage, the `Err` varient is returned two partial logs, a copy of this log and the
    /// given log. The first operation of  but whose first operations are blocking.
    ///
    /// Promised invarient: If two slices can be merged without blockages, they will be meaningfully the
    /// identical; however, identical sequences are not the same. For example, if player A records
    /// their match result and then player B records their result for their (different) match, the
    /// order of these can be swapped without issue.
    ///
    /// The algorithm: For each operation in the given slice, this slice is walked start to finish
    /// until one of the following happens.
    ///     1) An operation identical to the first event in the given log is found. This operation
    ///        is removed from both logs and push onto the new log. We then move to the next
    ///        operation in the given log.
    ///     2) An operation that blocks the first operation in the given log is found. The new log
    ///        is applied, and the current logs are returned to be rectified.
    ///     3) The end of the sliced log is reached and the first operation of the given log is
    ///        removed and push onto the new log.
    /// If there are remaining elements in the sliced log, those are removed and pushed onto the
    /// new log.
    /// The new log is then returned.
    ///
    /// Every operation "knows" what it blocks.
    pub fn merge(self, other: OpSlice) -> SyncStatus {
        let mut agreed: Vec<FullOp> = Vec::with_capacity(self.ops.len() + other.ops.len());
        let mut self_iter = self.ops.iter();
        let mut other_iter = self.ops.iter();
        while let Some(self_op) = self_iter.next() {
            // Our (the server's) rollbacks are ok
            if !self_op.active {
                agreed.push(self_op.clone());
                continue;
            }
            if let Some(other_op) = other_iter.next() {
                if !other_op.active {
                    // Their (the client's) rollbacks are not ok. They need to squash them or use
                    // our history.
                    return SyncStatus::SyncError(SyncError::RollbackFound(other));
                }
                if self_op.op == other_op.op {
                    agreed.push(self_op.clone());
                }
                for i_op in self_iter.clone() {
                    if i_op.op == other_op.op {
                        agreed.push(other_op.clone());
                        break;
                    } else if i_op.blocks(other_op) || other_op.blocks(i_op) {
                        // Blockage found!
                        return SyncStatus::InProgress(Blockage {
                            known: OpSlice {
                                ops: self_iter.cloned().collect(),
                            },
                            agreed: OpSlice { ops: agreed },
                            other: OpSlice {
                                ops: other_iter.cloned().collect(),
                            },
                            problem: (i_op.clone(), other_op.clone()),
                        });
                    }
                }
            } else {
                agreed.push(self_op.clone());
            }
        }
        SyncStatus::Completed(OpSync {
            ops: OpSlice { ops: agreed },
        })
    }
}

impl Default for OpSlice {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Rollback> for OpSlice {
    fn from(r: Rollback) -> OpSlice {
        r.ops
    }
}

impl From<OpSync> for OpSlice {
    fn from(s: OpSync) -> OpSlice {
        s.ops
    }
}

impl From<SyncError> for SyncStatus {
    fn from(other: SyncError) -> SyncStatus {
        SyncStatus::SyncError(other)
    }
}

impl From<Blockage> for SyncStatus {
    fn from(other: Blockage) -> SyncStatus {
        SyncStatus::InProgress(other)
    }
}

impl From<OpSync> for SyncStatus {
    fn from(other: OpSync) -> SyncStatus {
        SyncStatus::Completed(other)
    }
}

impl FullOp {
    /// Creates a new FullOp from an existing TournOp
    pub fn new(op: TournOp) -> Self {
        Self {
            op,
            id: OpId::new(Uuid::new_v4()),
            salt: Utc::now(),
            active: true,
        }
    }

    /// Determines if the given operation affects this operation
    pub fn blocks(&self, other: &Self) -> bool {
        self.op.blocks(&other.op)
    }
}

impl TournOp {
    /// Determines if this operation only makes sense if it happens before the other.
    pub fn implied_ordering(&self, _other: &Self) -> bool {
        todo!()
        /*
        use TournOp::*;
        match self {
            Freeze => other == &Thaw(),
            ReadyPlayer(p1) => if let UnReadyPlayer(p2) = other {
                p1 == p2
            } else {
                false
            },
            _ => false,
        }
        */
    }

    /// Determines if this operation blocks a given operation
    pub fn blocks(&self, _other: &Self) -> bool {
        todo!()
        /*
        use TournOp::*;
        match self {
            // Blocks everything
            Freeze |
            Thaw |
            End |
            Cancel => true,
            // Blocks nothing
            Create(_) |
            TimeExtension(RoundIdentifier, Duration) => false,
            // Blocks at least one thing
            CheckIn(p1) => match other {
                PrunePlayers => true,
                _ => false,
            },
            RegisterPlayer(_) => match other {
                PrunePlayers => true,
                Cut(usize) => false,
                ReadyPlayer(PlayerIdentifier) => false,
                UnReadyPlayer(PlayerIdentifier) => false,
            },
            UpdateReg(bool) => false,
            Start => false,
            RecordResult(RoundIdentifier, RoundResult) => false,
            ConfirmResult(PlayerIdentifier) => false,
            DropPlayer(PlayerIdentifier) => false,
            AdminDropPlayer(PlayerIdentifier) => false,
            AddDeck(PlayerIdentifier, String, Deck) => false,
            RemoveDeck(PlayerIdentifier, String) => false,
            RemoveRound(RoundIdentifier) => false,
            SetGamerTag(PlayerIdentifier, String) => false,
            ReadyPlayer(PlayerIdentifier) => false,
            UnReadyPlayer(PlayerIdentifier) => false,
            UpdateTournSetting(TournamentSetting) => false,
            GiveBye(PlayerIdentifier) => false,
            CreateRound(Vec<PlayerIdentifier>) => false,
            PairRound => false,
            Cut(usize) => false,
            PruneDecks => false,
            PrunePlayers => false,
        }
        */
    }
}

impl OpData {
    /// Calculates if the data is nothing
    pub fn is_nothing(&self) -> bool {
        match self {
            Self::Nothing => true,
            _ => false,
        }
    }

    /// Assumes contained data is `Nothing`
    ///
    /// PANICS: If the data is anything else, this method panics.
    pub fn assume_nothing(self) {
        match self {
            Self::Nothing => (),
            _ => panic!("Assumed OpData was nothing, failed"),
        }
    }

    /// Assumes contained data is from `RegisterPlayer` and returns that id, analogous to `unwrap`.
    ///
    /// PANICS: If the data is anything else, this method panics.
    pub fn assume_register_player(self) -> PlayerId {
        match self {
            Self::RegisterPlayer(id) => id,
            _ => panic!("Assumed OpData was register player, failed"),
        }
    }

    /// Assumes contained data is from `RegisterJudge` and returns that id, analogous to `unwrap`.
    ///
    /// PANICS: If the data is anything else, this method panics.
    pub fn assume_register_judge(self) -> Judge {
        match self {
            Self::RegisterJudge(judge) => judge,
            _ => panic!("Assumed OpData was register judge, failed"),
        }
    }

    /// Assumes contained data is from `RegisterAdmin` and returns that id, analogous to `unwrap`.
    ///
    /// PANICS: If the data is anything else, this method panics.
    pub fn assume_register_admin(self) -> Admin {
        match self {
            Self::RegisterAdmin(admin) => admin,
            _ => panic!("Assumed OpData was register admin, failed"),
        }
    }

    /// Assumes contained data is from `ConfirmResult` and returns that id, analogous to `unwrap`.
    ///
    /// PANICS: If the data is anything else, this method panics.
    pub fn assume_confirm_result(self) -> (RoundId, RoundStatus) {
        match self {
            Self::ConfirmResult(r_id, status) => (r_id, status),
            _ => panic!("Assumed OpData was confirm result, failed"),
        }
    }

    /// Assumes contained data is from `GiveBye` and returns that id, analogous to `unwrap`.
    ///
    /// PANICS: If the data is anything else, this method panics.
    pub fn assume_give_bye(self) -> RoundId {
        match self {
            Self::GiveBye(id) => id,
            _ => panic!("Assumed OpData was give bye, failed"),
        }
    }

    /// Assumes contained data is from `CreateRound` and returns that id, analogous to `unwrap`.
    ///
    /// PANICS: If the data is anything else, this method panics.
    pub fn assume_create_round(self) -> RoundId {
        match self {
            Self::CreateRound(id) => id,
            _ => panic!("Assumed OpData was create round, failed"),
        }
    }

    /// Assumes contained data is from `Pair` and returns that id, analogous to `unwrap`.
    ///
    /// PANICS: If the data is anything else, this method panics.
    pub fn assume_create_pairings(self) -> Pairings {
        match self {
            Self::CreatePairings(pairings) => pairings,
            _ => panic!("Assumed OpData was CreatePairings, failed."),
        }
    }

    /// Assumes contained data is from `Pair` and returns that id, analogous to `unwrap`.
    ///
    /// PANICS: If the data is anything else, this method panics.
    pub fn assume_pair(self) -> Vec<RoundId> {
        match self {
            Self::Pair(ids) => ids,
            _ => panic!("Assumed OpData was pair round, failed"),
        }
    }
}

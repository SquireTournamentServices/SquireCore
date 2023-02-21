use std::{
    collections::vec_deque::{Drain, IntoIter, Iter, VecDeque},
    slice,
};

use serde::{Deserialize, Serialize};

use crate::{
    model::{
        accounts::SquireAccount,
        tournament::{Tournament, TournamentSeed},
    },
    sync::{op_sync::OpSync, sync_error::SyncError, FullOp, OpId},
};

use super::{Rollback, RollbackError};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// An ordered list of all operations applied to a tournament
pub struct OpLog {
    pub(crate) owner: SquireAccount,
    pub(crate) seed: TournamentSeed,
    pub(crate) ops: Vec<FullOp>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// An ordered list of some of the operations applied to a tournament
pub struct OpSlice {
    pub(crate) ops: VecDeque<FullOp>,
}

impl OpLog {
    /// Creates a new log
    pub fn new(owner: SquireAccount, seed: TournamentSeed) -> Self {
        OpLog {
            owner,
            seed,
            ops: vec![],
        }
    }

    /// Creates the initial state of the tournament
    pub fn init_tourn(&self) -> Tournament {
        self.owner.create_tournament(self.seed.clone())
    }

    /// Calculates the length of inner `Vec` of `FullOp`s
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// Calculates if the length of inner `Vec` of `FullOp`s is empty
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Adds an operation to the end of the OpLog
    pub fn add_op(&mut self, op: FullOp) {
        self.ops.push(op);
    }

    /// Returns an iterator over the log
    pub fn iter(&self) -> slice::Iter<'_, FullOp> {
        self.ops.iter()
    }

    /// Splits the log into two halves. The first operation in the second half will have the same
    /// id as the given id (if that id can be found).
    pub fn split_at(&self, id: OpId) -> (OpSlice, OpSlice) {
        let index = self
            .ops
            .iter()
            .position(|op| id == op.id)
            .unwrap_or(self.ops.len());
        let (left, right) = self.ops.split_at(index);
        (
            left.iter().cloned().collect(),
            right.iter().cloned().collect(),
        )
    }

    /// Splits the log into two halves and returns the first half. The returned slice will stop at
    /// the last operation before given id, i.e. the slice will not contain the given operation.
    pub fn split_at_first(&self, id: OpId) -> OpSlice {
        let index = self
            .ops
            .iter()
            .position(|op| id == op.id)
            .unwrap_or(self.ops.len());
        let (digest, _) = self.ops.split_at(index);
        digest.iter().cloned().collect()
    }

    /// Splits the log into two halves. The first half is used to populate the tournament. The
    /// first operation in the given slice will have the same id as the given id (if that id can be
    /// found).
    pub fn split_at_tourn(&self, id: OpId) -> Result<Tournament, SyncError> {
        let mut tourn = self.init_tourn();
        let mut found = false;
        for op in self.ops.iter().cloned() {
            found |= id == op.id;
            if found {
                break;
            } else {
                // This should never error... but just in case
                tourn.apply_op(op.salt, op.op)?;
            }
        }
        if !found {
            return Err(SyncError::UnknownOperation(id));
        }
        Ok(tourn)
    }

    pub(crate) fn slice_up_to(&self, id: OpId) -> Option<OpSlice> {
        let mut found = false;
        let ops = self
            .ops
            .iter()
            .cloned()
            .take_while(|op| {
                found &= op.id == id;
                found
            })
            .collect();
        found.then_some(OpSlice { ops })
    }

    /// Creates a slice of this log starting at the given index. `None` is returned if `index` is
    /// out of bounds.
    pub(crate) fn get_slice(&self, id: OpId) -> Option<OpSlice> {
        self.get_slice_extra(id, 0)
    }

    /// Creates a slice of this log starting at the given index. `None` is returned if `index` is
    /// out of bounds.
    pub(crate) fn get_slice_extra(&self, id: OpId, extra: usize) -> Option<OpSlice> {
        let len = self.ops.len() - 1;
        let index = self
            .ops
            .iter()
            .rev()
            .enumerate()
            .find_map(|(i, op)| (op.id == id).then_some(len - i))?
            .checked_sub(extra)
            .unwrap_or_default();
        Some(OpSlice {
            ops: self.ops[index..].iter().cloned().collect(),
        })
    }

    pub(crate) fn slice_from_slice(&self, ops: &OpSlice) -> Result<OpSlice, SyncError> {
        let op = ops.start_op().ok_or(SyncError::EmptySync)?;
        let slice = self
            .get_slice(op.id)
            .ok_or_else(|| SyncError::UnknownOperation(op.id))?;
        match slice.start_op().unwrap() == op {
            true => Ok(slice),
            false => Err(SyncError::RollbackFound(op.id)),
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
            return Err(RollbackError::OutOfSync(OpSync {
                owner: self.owner.clone(),
                seed: self.seed.clone(),
                ops: slice,
            }));
        }
        let mut r_op = rollback.ops.ops.iter();
        for i_op in slice.ops.iter() {
            // If the id is unknown, the operation is unknow... so we continue.
            // Unknown, inactive ops ok to keep around. They can't affect anything
            r_op.by_ref().find(|r| i_op.id == r.id).ok_or_else(|| {
                RollbackError::OutOfSync(OpSync {
                    owner: self.owner.clone(),
                    seed: self.seed.clone(),
                    ops: slice.clone(),
                })
            })?;
        }
        // This should never return an Err
        self.overwrite(rollback.ops)
            .map_err(RollbackError::SliceError)
    }
}

impl OpSlice {
    /// Creates a new slice
    pub fn new() -> Self {
        OpSlice {
            ops: VecDeque::new(),
        }
    }

    /// Calculates the length of inner `Vec` of `FullOp`s
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// Calculates if the length of inner `Vec` of `FullOp`s is empty
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Adds an operation to the end of the OpSlice
    pub fn add_op(&mut self, op: FullOp) {
        self.ops.push_back(op);
    }

    /// Returns the index of the first stored operation.
    pub fn start_op(&self) -> Option<FullOp> {
        self.ops.front().cloned()
    }

    /// Returns the index of the first stored operation.
    pub fn start_id(&self) -> Option<OpId> {
        self.ops.front().map(|o| o.id)
    }

    /// Takes the slice and strips all inactive operations. This is only needed in the unlikely
    /// scenerio where a client rollbacks without communicating with the server and then tries to
    /// sync with the server.
    pub fn squash(self) -> Self {
        self.ops.into_iter().filter(|o| o.active).collect()
    }

    /// Splits the slice into two halves. The first operation in the second half will have the same
    /// id as the given id (if that id can be found).
    pub fn split_at(&self, id: OpId) -> (OpSlice, OpSlice) {
        let mut left = OpSlice::new();
        let mut right = OpSlice::new();
        let mut found = false;
        for op in self.ops.iter().cloned() {
            found |= id == op.id;
            if found {
                right.add_op(op);
            } else {
                left.add_op(op);
            }
        }
        (left, right)
    }

    pub fn iter(&self) -> Iter<'_, FullOp> {
        self.ops.iter()
    }

    pub fn drain(&mut self) -> Drain<'_, FullOp> {
        self.ops.drain(0..)
    }

    pub fn pop_front(&mut self) -> Option<FullOp> {
        self.ops.pop_front()
    }
}

impl IntoIterator for OpSlice {
    type Item = FullOp;

    type IntoIter = IntoIter<FullOp>;

    fn into_iter(self) -> Self::IntoIter {
        self.ops.into_iter()
    }
}

impl FromIterator<FullOp> for OpSlice {
    fn from_iter<I: IntoIterator<Item = FullOp>>(iter: I) -> Self {
        Self {
            ops: iter.into_iter().collect(),
        }
    }
}

impl Extend<FullOp> for OpSlice {
    fn extend<T: IntoIterator<Item = FullOp>>(&mut self, iter: T) {
        self.ops.extend(iter)
    }
}

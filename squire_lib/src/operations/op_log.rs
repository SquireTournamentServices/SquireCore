use serde::{Deserialize, Serialize};

use crate::{
    identifiers::OpId,
    operations::{Blockage, FullOp, OpSync, SyncError, SyncStatus},
};

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
/// An enum that encodes that errors that can occur during a rollback
pub enum RollbackError {
    /// The rollback slice has an unknown starting point
    SliceError(SyncError),
    /// The log that doesn't contain the rollback contains operations that the rolled back log
    /// doesn't contain
    OutOfSync(OpSync),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A struct used to communicate a rollback
pub struct Rollback {
    pub(crate) ops: OpSlice,
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
    pub(crate) fn get_slice_extra(&self, id: OpId, extra: usize) -> Option<OpSlice> {
        let index = self
            .ops
            .iter()
            .rev()
            .enumerate()
            .find_map(|(i, op)| (op.id == id).then_some(i))?
            .checked_sub(extra)
            .unwrap_or_default();
        Some(OpSlice {
            ops: self.ops[index..].to_vec(),
        })
    }

    pub(crate) fn slice_from_slice(&self, ops: &OpSlice) -> Result<OpSlice, SyncError> {
        let op = ops.start_op().ok_or(SyncError::EmptySync)?;
        let slice = self
            .get_slice(op.id)
            .ok_or_else(|| SyncError::UnknownOperation(op.clone()))?;
        match slice.start_op().unwrap() != op {
            true => Ok(slice),
            false => Err(SyncError::RollbackFound(slice)),
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
        match self.slice_from_slice(&other.ops) {
            Ok(slice) => slice.merge(other.ops),
            Err(e) => SyncStatus::SyncError(e),
        }
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
    /// given log.
    ///
    /// Promised invarient: If two slices can be merged without blockages, they will be meaningfully the
    /// identical; however, this does not mean they are identical sequences. For example, if player A records
    /// their match result and then player B records their result for their (different) match, the
    /// order of these can be swapped without issue.
    ///
    /// The algorithm: For each operation in the given slice, this slice is walked start to finish
    /// until one of the following happens.
    ///     1) An identical operation in this log is found. This operation is removed from both
    ///        logs and push onto the new log. We then move to the next operation in the given log.
    ///         a) This is only true if both operations are the first in their logs
    ///     2) An operation that blocks the operation is found. The problematic operations are
    ///        removed and returned along with the partial logs.
    ///     3) The end of this log is reached and this operation is removed and pushed onto the new
    ///        log.
    ///
    /// If there are remaining elements in the sliced log, those are removed and pushed onto the
    /// new log.
    /// The new log is then returned.
    ///
    /// Every operation "knows" what it blocks.
    pub fn merge(mut self, other: OpSlice) -> SyncStatus {
        let mut agreed: Vec<FullOp> = Vec::with_capacity(other.ops.len());
        let mut iter = other.ops.into_iter();
        while let Some(op) = iter.next() {
            // 1
            if self.ops.first().unwrap().op == op.op {
                agreed.push(self.ops.remove(0));
                continue;
            }
            match self
                .ops
                .iter()
                .enumerate()
                .find_map(|(i, sop)| sop.blocks(&op).then_some(i))
            {
                Some(index) => {
                    /* 2 */
                    let block = self.ops.remove(index);
                    return SyncStatus::InProgress(Blockage {
                        known: self,
                        agreed: OpSlice { ops: agreed },
                        other: OpSlice {
                            ops: iter.collect(),
                        },
                        problem: (block, op),
                    });
                }
                None => {
                    /* 3 */
                    agreed.push(op);
                }
            }
        }
        // All the agreed upon operations happen after this log
        self.ops.extend(agreed);
        SyncStatus::Completed(OpSync { ops: self })
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

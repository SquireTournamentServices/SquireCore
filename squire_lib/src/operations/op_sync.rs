use serde::{Deserialize, Serialize};

use crate::operations::{FullOp, OpSlice};

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
    /// The logs have been successfully synced
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
/// A struct to help resolve blockages
pub struct Blockage {
    pub(crate) known: OpSlice,
    pub(crate) agreed: OpSlice,
    pub(crate) other: OpSlice,
    pub(crate) problem: (FullOp, FullOp),
}

impl Blockage {
    /// Returns the problematic pair of operations.
    pub fn problem(&self) -> &(FullOp, FullOp) {
        &self.problem
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
        let (p_one, p_two) = self.problem.clone();
        if first == p_one {
            self.agreed.add_op(p_one);
            self.agreed.add_op(p_two);
        } else if first == p_two {
            self.agreed.add_op(p_two);
            self.agreed.add_op(p_one);
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

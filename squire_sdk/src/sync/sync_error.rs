use std::{ops::FromResidual, convert::Infallible};

use serde::{Deserialize, Serialize};

use crate::model::error::TournamentError;

use super::{OpId, processor::SyncProblem};

/// An enum that captures errors with the validity of sync requests.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum SyncError {
    /// At least one of the logs was empty
    EmptySync,
    /// The `OpSync` was a mismatch for the tournament manager (e.g. wrong account or seed)
    InvalidRequest,
    /// One of the logs contains a rollback that the other doesn't have
    RollbackFound(OpId),
    /// The starting operation of the slice in unknown to the other log
    UnknownOperation(OpId),
    /// An error in the tournament occured (this should not happen)
    TournamentError(TournamentError),
}

/// An enum that captures issues found during the processing of a validated sync process
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum MergeError {
    /// There is not simple way to solve the merge conflict. The user must take handle the merge
    /// manually. This most frequently happens when a merge creates tournament errors
    Irreconcilable,
    /// The given log was either invalid or did not match the sync process
    InvalidLog,
    /// This error occurs when two slices can't be placed in arbitary order and the user must
    /// handle merging them.
    Incompatable(Box<SyncProblem>),
}

impl From<TournamentError> for MergeError {
    fn from(_: TournamentError) -> Self {
        Self::Irreconcilable
    }
}

impl FromResidual<Result<Infallible, TournamentError>> for MergeError {
    fn from_residual(residual: Result<Infallible, TournamentError>) -> Self {
        match residual {
            Ok(_) => unreachable!("Infallible"),
            Err(_) => Self::Irreconcilable,
        }
    }
}

impl From<SyncError> for MergeError {
    fn from(_: SyncError) -> Self {
        Self::InvalidLog
    }
}

impl FromResidual<Result<Infallible, SyncError>> for MergeError {
    fn from_residual(residual: Result<Infallible, SyncError>) -> Self {
        match residual {
            Ok(_) => unreachable!("Infallible"),
            Err(_) => Self::InvalidLog,
        }
    }
}

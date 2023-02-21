use std::{convert::Infallible, ops::FromResidual};

use serde::{Deserialize, Serialize};

use crate::sync::{op_sync::OpSync, sync_error::SyncError};

use super::{processor::SyncProcessor, MergeError};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// An enum to help track the progress of the syncing of two op logs
pub enum SyncStatus {
    /// There was an error when attempting to initially sync
    SyncError(Box<SyncError>),
    /// There are discrepancies in between the two logs that are being synced
    InProgress(Box<(SyncProcessor, MergeError)>),
    /// The logs have been successfully synced
    Completed(OpSync),
}

impl SyncStatus {
    /// Calculates if the status is an error
    pub fn is_error(&self) -> bool {
        matches!(self, SyncStatus::SyncError(_))
    }

    /// Calculates if the status is a blockage
    pub fn is_in_progress(&self) -> bool {
        matches!(self, SyncStatus::InProgress(_))
    }

    /// Calculates if the status is a success
    pub fn is_completed(&self) -> bool {
        matches!(self, SyncStatus::Completed(_))
    }

    /// Comsumes self and returns the held error if `self` is `SyncError` and panics otherwise
    pub fn assume_error(self) -> Box<SyncError> {
        match self {
            SyncStatus::SyncError(err) => err,
            SyncStatus::InProgress(block) => {
                panic!("Sync status was not an error but was a blockage: {block:?}")
            }
            SyncStatus::Completed(sync) => {
                panic!("Sync status was not an error but was completed: {sync:?}")
            }
        }
    }

    /// Comsumes self and returns the held error if `self` is `InProgress` and panics otherwise
    pub fn assume_in_progress(self) -> Box<(SyncProcessor, MergeError)> {
        match self {
            SyncStatus::InProgress(prog) => prog,
            SyncStatus::SyncError(err) => {
                panic!("Sync status was not a blockage but was an error: {err:?}")
            }
            SyncStatus::Completed(sync) => {
                panic!("Sync status was not a blockage but was completed: {sync:?}")
            }
        }
    }

    /// Comsumes self and returns the held error if `self` is `Complete` and panics otherwise
    pub fn assume_completed(self) -> OpSync {
        match self {
            SyncStatus::Completed(sync) => sync,
            SyncStatus::InProgress(block) => {
                panic!("Sync status was not completed but was a blockage: {block:?}")
            }
            SyncStatus::SyncError(err) => {
                panic!("Sync status was not completed but was an error: {err:?}")
            }
        }
    }
}

impl<E> FromResidual<Result<Infallible, E>> for SyncStatus
where
    E: Into<Box<SyncError>>,
{
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        match residual {
            Ok(_) => unreachable!(""),
            Err(err) => Self::SyncError(err.into()),
        }
    }
}

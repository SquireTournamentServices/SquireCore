
impl From<SyncError> for SyncStatus {
    fn from(other: SyncError) -> SyncStatus {
        SyncStatus::SyncError(Box::new(other))
    }
}

impl From<Box<SyncError>> for SyncStatus {
    fn from(other: Box<SyncError>) -> SyncStatus {
        SyncStatus::SyncError(other)
    }
}

impl From<Blockage> for SyncStatus {
    fn from(other: Blockage) -> SyncStatus {
        SyncStatus::InProgress(Box::new(other))
    }
}

impl From<Box<Blockage>> for SyncStatus {
    fn from(other: Box<Blockage>) -> SyncStatus {
        SyncStatus::InProgress(other)
    }
}

impl From<OpSync> for SyncStatus {
    fn from(other: OpSync) -> SyncStatus {
        SyncStatus::Completed(other)
    }
}


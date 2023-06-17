use serde::{Deserialize, Serialize};

use super::{FullOp, OpId, OpLog, OpSlice, OpSync, SyncError};

/// This type results from a client making a decision about what operations need to stay and what
/// operations need to be removed from its log during the sync process.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum SyncDecision {
    /// The client decided to ignore the problematic operation and is asking the backend to
    /// continue trying to process things.
    Plucked(SyncProcessor),
    /// The client decided to ignore the rest of its log. This will mark the processing as done.
    Purged(SyncCompletion),
}

/// This type encodes the result of a successful sync.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum SyncCompletion {
    /// The backend did not have any operations that were unknown to the client.
    ForeignOnly(OpSlice),
    /// The backend had one or more operations that were unknown to the client and the logs were
    /// successfully merged.
    Mixed(OpSlice),
}

impl SyncCompletion {
    pub fn len(&self) -> usize {
        match self {
            Self::ForeignOnly(ops) | Self::Mixed(ops) => ops.len(),
        }
    }
}

/// This struct contain an in-progress sync. The processor is mostly used internally by the
/// `TournamentManager` to process sync requests; however, it is also shared during the sync
/// process so that the client can audit its log. Those methods produce an `OpDecision`.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct SyncProcessor {
    pub(crate) known: OpSlice,
    pub(crate) processed: OpSlice,
    pub(crate) to_process: OpSlice,
}

impl SyncProcessor {
    /// This method is called by the client to decided how to process errors in the sync process.
    /// This method simply removes the operation that is causing the problem, preserving both the
    /// rest of its log
    pub fn pluck(mut self) -> SyncDecision {
        self.to_process.pop_front();
        SyncDecision::Plucked(self)
    }
    
    pub fn left(&self) -> 

    /// This method is called by the client to decided how to process errors in the sync process.
    /// This method removes all of the remaining operations that it is trying to sync.
    pub fn purge(self) -> SyncDecision {
        SyncDecision::Purged(self.finalize())
    }

    /// Returns whether or not the process will merge two logs or just one
    pub fn is_foreign_only(&self) -> bool {
        self.known.len() <= 1
    }

    /// Creates a new processor from an `OpSync` and an `OpLog`. This method is fallible for a
    /// number of reasons, the sync could be a mismatch with the log, the sync could be empty, and
    /// the sync might have an incorrect anchor operation.
    pub(crate) fn new(mut sync: OpSync, log: &OpLog) -> Result<Self, SyncError> {
        sync.validate(log)?;
        let id = sync.first_id()?;
        let known = match log.get_slice(id) {
            Some(slice) => {
                // Remove the anchor operation from the to_process slice
                let _ = sync.pop_front()?;
                slice
            }
            None if log.is_empty() => OpSlice::new(),
            _ => return Err(SyncError::UnknownOperation(id)),
        };
        Ok(Self {
            known,
            processed: OpSlice::new(),
            to_process: sync.ops,
        })
    }

    pub(crate) fn last_known(&self) -> Option<OpId> {
        self.known.last_id()
    }

    pub(crate) fn move_all(&mut self) {
        self.processed.extend(self.to_process.drain())
    }

    pub(crate) fn finalize(mut self) -> SyncCompletion {
        if self.is_foreign_only() {
            self.known.extend(self.processed);
            SyncCompletion::ForeignOnly(self.known)
        } else {
            self.known.extend(self.processed);
            SyncCompletion::Mixed(self.known)
        }
    }

    pub(crate) fn processing<'a>(&'a mut self) -> Processing<'a> {
        Processing::new(&mut *self)
    }
}

pub struct Processing<'a> {
    proc: &'a mut SyncProcessor,
    processed: OpSlice,
    to_process: OpSlice,
    limbo: Option<FullOp>,
}

impl<'a> Processing<'a> {
    fn new(proc: &'a mut SyncProcessor) -> Processing<'a> {
        let mut to_process = proc.processed.clone();
        to_process.extend(proc.to_process.iter().cloned());
        Self {
            proc,
            processed: OpSlice::new(),
            limbo: None,
            to_process,
        }
    }
}

impl Iterator for Processing<'_> {
    type Item = FullOp;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the first operation that is to be processed
        let digest = self.to_process.pop_front()?;
        // Place that operation into the limbo state. If processing fails, we know that this
        // operation is the reason for the failure. The prior operation is safe, so we can put it
        // in `processed`.
        if let Some(ok_op) = self.limbo.replace(digest.clone()) {
            self.processed.add_op(ok_op);
        }
        Some(digest)
    }
}

impl ExactSizeIterator for Processing<'_> {
    fn len(&self) -> usize {
        self.to_process.len()
    }
}

impl Drop for Processing<'_> {
    fn drop(&mut self) {
        if self.to_process.is_empty() {
            self.proc.move_all()
        } else {
            if let Some(op) = self.limbo.take() {
                self.to_process.ops.push_front(op);
            }
            self.proc.processed.clear();
            self.proc.to_process.clear();
            self.proc.processed.extend(self.processed.drain());
            self.proc.to_process.extend(self.to_process.drain());
        }
    }
}

#[cfg(test)]
mod tests {
    use squire_lib::operations::TournOp;
    use squire_tests::spoof_account;

    use crate::sync::{FullOp, OpSlice};

    use super::SyncProcessor;

    fn spoof_op() -> FullOp {
        FullOp::new(TournOp::RegisterPlayer(spoof_account()))
    }

    fn spoof_ops(count: usize) -> impl IntoIterator<Item = FullOp> {
        std::iter::repeat_with(spoof_op).take(count)
    }

    fn spoof_foreign_proc(count: usize) -> SyncProcessor {
        SyncProcessor {
            known: OpSlice::from_iter(spoof_ops(1)),
            processed: OpSlice::new(),
            to_process: OpSlice::from_iter(spoof_ops(count)),
        }
    }

    fn spoof_mixed_proc(count: usize) -> SyncProcessor {
        SyncProcessor {
            known: OpSlice::from_iter(spoof_ops(2)),
            processed: OpSlice::new(),
            to_process: OpSlice::from_iter(spoof_ops(count)),
        }
    }

    fn spoof_in_progress_proc(count: usize) -> SyncProcessor {
        SyncProcessor {
            known: OpSlice::from_iter(spoof_ops(1)),
            processed: OpSlice::from_iter(spoof_ops(1)),
            to_process: OpSlice::from_iter(spoof_ops(count)),
        }
    }

    fn spoof_in_progress_mixed_proc(count: usize) -> SyncProcessor {
        SyncProcessor {
            known: OpSlice::from_iter(spoof_ops(1)),
            processed: OpSlice::from_iter(spoof_ops(1)),
            to_process: OpSlice::from_iter(spoof_ops(count)),
        }
    }

    #[test]
    fn iter_all_tests() {
        // Foreign only
        let mut proc = spoof_foreign_proc(5);
        assert_eq!(proc.to_process.len(), 5);
        assert!(proc.processed.is_empty());
        proc.processing().for_each(drop);
        assert!(proc.to_process.is_empty());
        assert_eq!(proc.processed.len(), 5);

        // Mixed only
        let mut proc = spoof_mixed_proc(5);
        assert_eq!(proc.to_process.len(), 5);
        assert!(proc.processed.is_empty());
        proc.processing().for_each(drop);
        assert!(proc.to_process.is_empty());
        assert_eq!(proc.processed.len(), 5);

        // Foreign in-progress only
        let mut proc = spoof_in_progress_proc(5);
        assert_eq!(proc.to_process.len(), 5);
        assert_eq!(proc.processed.len(), 1);
        proc.processing().for_each(drop);
        assert!(proc.to_process.is_empty());
        assert_eq!(proc.processed.len(), 6);

        // Mixed in-progress only
        let mut proc = spoof_in_progress_mixed_proc(5);
        assert_eq!(proc.to_process.len(), 5);
        assert_eq!(proc.processed.len(), 1);
        proc.processing().for_each(drop);
        assert!(proc.to_process.is_empty());
        assert_eq!(proc.processed.len(), 6);
    }

    #[test]
    fn drop_tests() {
        // Foreign only
        let mut proc = spoof_foreign_proc(5);
        assert_eq!(proc.to_process.len(), 5);
        assert!(proc.processed.is_empty());
        let _ = proc.processing().take(2).for_each(drop);
        assert_eq!(proc.to_process.len(), 4);
        assert_eq!(proc.processed.len(), 1);

        // Mixed only
        let mut proc = spoof_mixed_proc(5);
        assert_eq!(proc.to_process.len(), 5);
        assert!(proc.processed.is_empty());
        let _ = proc.processing().take(2).for_each(drop);
        assert_eq!(proc.to_process.len(), 4);
        assert_eq!(proc.processed.len(), 1);

        // Foreign in-progress only
        let mut proc = spoof_in_progress_proc(5);
        assert_eq!(proc.to_process.len(), 5);
        assert_eq!(proc.processed.len(), 1);
        let _ = proc.processing().take(3).for_each(drop);
        assert_eq!(proc.to_process.len(), 4);
        assert_eq!(proc.processed.len(), 2);

        // Mixed in-progress only
        let mut proc = spoof_in_progress_mixed_proc(5);
        assert_eq!(proc.to_process.len(), 5);
        assert_eq!(proc.processed.len(), 1);
        let _ = proc.processing().take(3).for_each(drop);
        assert_eq!(proc.to_process.len(), 4);
        assert_eq!(proc.processed.len(), 2);
    }
}

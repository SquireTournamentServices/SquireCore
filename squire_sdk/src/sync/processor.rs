use serde::{Deserialize, Serialize};

use super::OpSlice;
#[cfg(feature = "server")]
use super::{FullOp, OpId};
#[cfg(any(feature = "client", feature = "server"))]
use super::{OpLog, OpSync, SyncError};

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

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_slice(self) -> OpSlice {
        match self {
            SyncCompletion::ForeignOnly(ops) | SyncCompletion::Mixed(ops) => ops,
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

    /// Calculates number of foreign operations
    pub fn len(&self) -> usize {
        self.processed.len() + self.to_process.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

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
    #[cfg(any(feature = "client", feature = "server"))]
    pub(crate) fn new(mut sync: OpSync, log: &OpLog) -> Result<Self, SyncError> {
        sync.validate(log)?;
        let id = sync.first_id()?;
        let known = match log.get_slice(id) {
            Some(slice) => {
                // Remove the anchor operation from the to_process slice
                let _ = sync.pop_front()?;
                // Iterate through the slice and the sync, front to back, and extend all shared ops
                let mut iter = slice.iter();
                iter.next();
                for op in iter {
                    match sync.first_id() {
                        Ok(id) if id == op.id => {
                            println!("Found a matching Op: {id}! Dropping it from the proc!");
                            let _ = sync.pop_front();
                        }
                        Ok(_) | Err(_) => break,
                    }
                }
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

    #[cfg(feature = "server")]
    pub(crate) fn last_known(&self) -> Option<OpId> {
        self.known.last_id()
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

    #[cfg(feature = "server")]
    pub(crate) fn processing(&mut self) -> Processing<'_> {
        Processing::new(&mut *self)
    }
}

#[cfg(feature = "server")]
pub struct Processing<'a> {
    proc: &'a mut SyncProcessor,
    processed: OpSlice,
    to_process: OpSlice,
    limbo: Option<FullOp>,
}

#[cfg(feature = "server")]
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

    pub(crate) fn conclude(mut self) {
        if let Some(op) = self.limbo.take() {
            self.processed.add_op(op);
        }
    }
}

#[cfg(feature = "server")]
impl Iterator for &mut Processing<'_> {
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

#[cfg(feature = "server")]
impl ExactSizeIterator for &mut Processing<'_> {
    fn len(&self) -> usize {
        self.to_process.len()
    }
}

#[cfg(feature = "server")]
impl Drop for Processing<'_> {
    fn drop(&mut self) {
        if let Some(op) = self.limbo.take() {
            self.to_process.ops.push_front(op);
        }
        self.proc.processed.clear();
        self.proc.to_process.clear();
        self.proc.processed.extend(self.processed.drain());
        self.proc.to_process.extend(self.to_process.drain());
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use squire_lib::operations::TournOp;
    use squire_tests::spoof_account;

    use super::SyncProcessor;
    use crate::sync::{FullOp, OpSlice};

    fn spoof_op() -> FullOp {
        FullOp::new(TournOp::RegisterPlayer(spoof_account(), None))
    }

    fn spoof_ops(count: usize) -> impl IntoIterator<Item = FullOp> {
        std::iter::repeat_with(spoof_op).take(count)
    }

    fn spoof_foreign_proc(count: usize) -> SyncProcessor {
        let digest = SyncProcessor {
            known: OpSlice::from_iter(spoof_ops(1)),
            processed: OpSlice::new(),
            to_process: OpSlice::from_iter(spoof_ops(count)),
        };
        assert_eq!(digest.to_process.len(), count);
        assert!(digest.processed.is_empty());
        digest
    }

    fn spoof_mixed_proc(count: usize) -> SyncProcessor {
        let digest = SyncProcessor {
            known: OpSlice::from_iter(spoof_ops(2)),
            processed: OpSlice::new(),
            to_process: OpSlice::from_iter(spoof_ops(count)),
        };
        assert_eq!(digest.to_process.len(), count);
        assert!(digest.processed.is_empty());
        digest
    }

    fn spoof_in_progress_proc(count: usize) -> SyncProcessor {
        let digest = SyncProcessor {
            known: OpSlice::from_iter(spoof_ops(1)),
            processed: OpSlice::from_iter(spoof_ops(1)),
            to_process: OpSlice::from_iter(spoof_ops(count)),
        };
        assert_eq!(digest.to_process.len(), count);
        assert_eq!(digest.processed.len(), 1);
        digest
    }

    fn spoof_in_progress_mixed_proc(count: usize) -> SyncProcessor {
        let digest = SyncProcessor {
            known: OpSlice::from_iter(spoof_ops(1)),
            processed: OpSlice::from_iter(spoof_ops(1)),
            to_process: OpSlice::from_iter(spoof_ops(count)),
        };
        assert_eq!(digest.to_process.len(), count);
        assert_eq!(digest.processed.len(), 1);
        digest
    }

    fn process<F>(count: usize, f: F)
    where
        F: Fn(SyncProcessor),
    {
        println!("Testing foreign only...");
        f(spoof_foreign_proc(count));

        println!("Testing mixed only...");
        f(spoof_mixed_proc(count));

        println!("Testing foreign in-progress only...");
        f(spoof_in_progress_proc(count));

        println!("Mixed in-progress only...");
        f(spoof_in_progress_mixed_proc(count));
    }

    #[test]
    fn iter_all_tests() {
        process(5, |mut proc| {
            let len = proc.len();
            let mut iter = proc.processing();
            iter.for_each(drop);
            iter.conclude();
            assert!(proc.to_process.is_empty());
            assert_eq!(proc.processed.len(), len);
        })
    }

    #[test]
    fn drop_tests() {
        process(5, |mut proc| {
            let len = proc.len();
            proc.processing().take(1).for_each(drop);
            assert_eq!(proc.to_process.len(), len);
            assert!(proc.processed.is_empty());
            proc.processing().take(3).for_each(drop);
            assert_eq!(proc.to_process.len(), len - 2);
            assert_eq!(proc.processed.len(), 2);
        })
    }

    #[test]
    fn handle_final_op() {
        process(5, |mut proc| {
            let len = proc.len();
            proc.processing().for_each(drop);
            assert_eq!(proc.to_process.len(), 1);
            assert_eq!(proc.processed.len(), len - 1);
            let mut iter = proc.processing();
            iter.for_each(drop);
            iter.conclude();
            assert!(proc.to_process.is_empty());
            assert_eq!(proc.processed.len(), len);
        })
    }
}

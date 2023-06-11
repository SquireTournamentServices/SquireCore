use serde::{Deserialize, Serialize};
use squire_lib::{error::TournamentError, tournament::Tournament};

use super::{FullOp, OpLog, OpSlice, OpSync, ServerOpLink, SyncError};

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

/// This struct contain an in-progress sync. The processor is mostly used internally by the
/// `TournamentManager` to process sync requests; however, it is also shared during the sync
/// process so that the client can audit its log. Those methods produce an `OpDecision`.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct SyncProcessor {
    /// Whether or not sync contains only foreign operations
    foreign_only: bool,
    known: OpSlice,
    to_process: OpSlice,
}

impl SyncProcessor {
    /// This method is called by the client to decided how to process errors in the sync process.
    /// This method simply removes the operation that is causing the problem, preserving both the
    /// rest of its log
    pub fn pluck(mut self) -> SyncDecision {
        self.to_process.pop_front();
        SyncDecision::Plucked(self)
    }

    /// This method is called by the client to decided how to process errors in the sync process.
    /// This method removes all of the remaining operations that it is trying to sync.
    pub fn purge(self) -> SyncDecision {
        let comp = if self.is_foreign_only() {
            SyncCompletion::ForeignOnly(self.known)
        } else {
            SyncCompletion::Mixed(self.known)
        };
        SyncDecision::Purged(comp)
    }

    /// Returns whether or not the process will merge two logs or one
    pub fn is_foreign_only(&self) -> bool {
        self.foreign_only
    }

    /// Creates a new processor from an `OpSync` and an `OpLog`. This method is fallible for a
    /// number of reasons, the sync could be a mismatch with the log, the sync could be empty, and
    /// the sync might have an incorrect anchor operation.
    pub(crate) fn new(sync: OpSync, log: &OpLog) -> Result<Self, SyncError> {
        sync.validate(log)?;
        let id = sync.first_id()?;
        let Some(known) = log.get_slice(id) else { return Err(SyncError::UnknownOperation(id)) };
        let foreign_only = known.len() == 1;
        Ok(Self {
            foreign_only,
            known,
            to_process: sync.ops,
        })
    }

    /// Processes the sync request by simulating different possible tournament histories. The log
    /// is used to recreate the initial tournament history.
    pub(crate) fn process(self, log: &OpLog) -> ServerOpLink {
        /*
        let mut tourn = log.split_at_tourn(self.start_at)?;

        // This shouldn't error. If it does, it is likely that the wrong log was passed in
        apply_ops(&mut tourn, self.agreed.iter().cloned())?;

        while let Some(alignment) = self.align.next() {
            match alignment {
                OpAlignment::Agreed(op) => {
                    tourn.apply_op(op.salt, op.op.clone())?;
                    self.agreed.add_op(*op);
                }
                OpAlignment::ToMerge((known, foriegn)) => {
                    self.process_slices(&mut tourn, known, foriegn)?;
                    // process_slices adds the slices to self.agreed
                }
            }
        }
        Ok(())
        */
        todo!()
    }

    /*
    fn process_slices(
        &mut self,
        tourn: &mut Tournament,
        known: OpSlice,
        foreign: OpSlice,
    ) -> Result<(), MergeError> {
        // FIXME: We need for then just tournament errors. The errors here need to include the
        // slices so that the context can be completely re-constructed

        // Apply foriegn then known
        let mut f_then_k = tourn.clone();
        apply_ops(&mut f_then_k, foreign.iter().cloned())?;
        apply_ops(&mut f_then_k, known.iter().cloned())?;

        // Apply known then foriegn
        apply_ops(tourn, known.iter().cloned())?; // This shouldn't error
        let mut k_then_f = tourn.clone();
        apply_ops(&mut k_then_f, foreign.iter().cloned())?;
        if f_then_k == k_then_f {
            Ok(())
        } else {
            Err(MergeError::Incompatable(Box::new(SyncProblem {
                known,
                foreign,
            })))
        }
    }
    */
}

fn apply_ops<I: Iterator<Item = FullOp>>(
    tourn: &mut Tournament,
    ops: I,
) -> Result<(), TournamentError> {
    for op in ops {
        tourn.apply_op(op.salt, op.op)?;
    }
    Ok(())
}

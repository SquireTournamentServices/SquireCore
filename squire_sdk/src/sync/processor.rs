use serde::{Deserialize, Serialize};
use squire_lib::{error::TournamentError, tournament::Tournament};

use super::{
    op_align::{OpAlign, OpAlignment},
    FullOp, MergeError, OpId, OpLog, OpSlice, OpSync, SyncError,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct SyncProcessor {
    start_at: OpId,
    align: OpAlign,
    agreed: OpSlice,
}

impl SyncProcessor {
    pub(crate) fn new(sync: OpSync, log: &OpLog) -> Result<Self, SyncError> {
        let start_at = sync.first_id()?;
        let known = log.split_at_first(start_at);
        let align = OpAlign::new(known, sync.ops)?;
        Ok(Self {
            start_at,
            align,
            agreed: OpSlice::new(),
        })
    }

    /// Processes the sync request by simulating different possible tournament histories. The log
    /// is used to recreate the initial tournament history.
    pub(crate) fn process(&mut self, log: &OpLog) -> Result<(), MergeError> {
        let mut tourn = log.split_at_tourn(self.start_at)?;

        // This shouldn't error. If it does, it is likely that the wrong log was passed in
        apply_ops(&mut tourn, self.agreed.iter().cloned())?;

        while let Some(alignment) = self.align.next() {
            match alignment {
                OpAlignment::Agreed(op) => {
                    tourn.apply_op(op.salt, op.op.clone())?;
                    self.agreed.add_op(op);
                }
                OpAlignment::ToMerge((known, foriegn)) => {
                    self.process_slices(&mut tourn, known, foriegn)?;
                    // process_slices adds the slices to self.agreed
                }
            }
        }
        Ok(())
    }

    /// Processes the sync request by simulating different possible tournament histories. The log
    /// is used to recreate the initial tournament history.
    pub(crate) fn add_agreed_and_process(&mut self, ops: OpSlice, log: &OpLog) -> Result<(), MergeError> {
        let mut tourn = log.split_at_tourn(self.start_at)?;

        // This shouldn't error. If it does, it is likely that the wrong log was passed in
        apply_ops(&mut tourn, self.agreed.iter().cloned())?;

        // Before adding in the new operations, we must verify that they don't cause problems
        apply_ops(&mut tourn, ops.iter().cloned())?;
        self.agreed.extend(ops);

        while let Some(alignment) = self.align.next() {
            match alignment {
                OpAlignment::Agreed(op) => {
                    tourn.apply_op(op.salt, op.op.clone())?;
                    self.agreed.add_op(op);
                }
                OpAlignment::ToMerge((known, foriegn)) => {
                    self.process_slices(&mut tourn, known, foriegn)?;
                    // process_slices adds the slices to self.agreed
                }
            }
        }
        Ok(())
    }

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

    pub(crate) fn iter_known(&self) -> impl Iterator<Item = &'_ FullOp> {
        self.agreed.iter().chain(self.align.iter_known())
    }

    pub(crate) fn iter_foreign(&self) -> impl Iterator<Item = &'_ FullOp> {
        self.agreed.iter().chain(self.align.iter_foreign())
    }
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct SyncProblem {
    pub known: OpSlice,
    pub foreign: OpSlice,
}

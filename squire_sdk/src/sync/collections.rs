use std::{
    cell::RefCell,
    collections::vec_deque::{self, Drain, IntoIter, Iter, VecDeque},
    slice,
};

use serde::{Deserialize, Serialize};

use squire_lib::{operations::OpUpdate, players::PlayerId, rounds::RoundId};

use crate::{
    model::{
        accounts::SquireAccount,
        tournament::{Tournament, TournamentSeed},
    },
    sync::{error::SyncError, FullOp, OpId},
};

use super::{OpDiff, OpSync};

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
    pub(crate) fn new(owner: SquireAccount, seed: TournamentSeed) -> Self {
        OpLog {
            owner,
            seed,
            ops: vec![],
        }
    }

    pub(crate) fn create_sync_request(&self, op: Option<OpId>) -> OpSync {
        let ops = match op {
            Some(id) => self.get_slice(id).unwrap(),
            None => self.ops.iter().cloned().collect(),
        };
        OpSync {
            owner: self.owner.clone(),
            seed: self.seed.clone(),
            ops,
        }
    }

    /// Creates the initial state of the tournament
    pub(crate) fn init_tourn(&self) -> Tournament {
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

    /// Returns an iterator over the log
    pub(crate) fn iter(&self) -> slice::Iter<'_, FullOp> {
        self.ops.iter()
    }

    /// Splits the log into two halves and returns the first half. The returned slice will stop at
    /// the last operation before given id, i.e. the slice will not contain the given operation.
    pub(crate) fn split_at_first(&self, id: OpId) -> OpSlice {
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
    pub(crate) fn split_at_tourn(&self, id: OpId) -> Result<Tournament, SyncError> {
        /*
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
        */
        todo!()
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
        /*
        let op = ops.start_op().ok_or(SyncError::EmptySync)?;
        let slice = self
            .get_slice(op.id)
            .ok_or(SyncError::UnknownOperation(op.id))?;
        match slice.start_op().unwrap() == op {
            true => Ok(slice),
            false => Err(SyncError::RollbackFound(op.id)),
        }
        */
        todo!()
    }

    /// Removes all elements in the log starting at the first index of the given slice. All
    /// operations in the slice are then appended to the end of the log.
    pub(crate) fn overwrite(&mut self, ops: OpSlice) -> Result<(), SyncError> {
        let slice = self.slice_from_slice(&ops)?;
        let id = slice.start_id().unwrap();
        let index = self.ops.iter().position(|o| o.id == id).unwrap();
        self.ops.truncate(index);
        self.ops.extend(ops.ops.into_iter());
        Ok(())
    }
}

impl OpSlice {
    /// Creates a new slice
    pub(crate) fn new() -> Self {
        OpSlice {
            ops: VecDeque::new(),
        }
    }

    /// Calculates the length of inner `Vec` of `FullOp`s
    pub(crate) fn len(&self) -> usize {
        self.ops.len()
    }

    /// Calculates if the length of inner `Vec` of `FullOp`s is empty
    pub(crate) fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Adds an operation to the end of the OpSlice
    pub(crate) fn add_op(&mut self, op: FullOp) {
        self.ops.push_back(op);
    }

    /// Returns the index of the first stored operation.
    pub(crate) fn start_op(&self) -> Option<FullOp> {
        self.ops.front().cloned()
    }

    /// Returns the index of the first stored operation.
    pub(crate) fn start_id(&self) -> Option<OpId> {
        self.ops.front().map(|o| o.id)
    }

    /// Splits the slice into two halves. The first operation in the second half will have the same
    /// id as the given id (if that id can be found).
    pub(crate) fn split_at(&self, id: OpId) -> (OpSlice, OpSlice) {
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

    pub(crate) fn iter(&self) -> Iter<'_, FullOp> {
        self.ops.iter()
    }

    pub(crate) fn drain(&mut self) -> Drain<'_, FullOp> {
        self.ops.drain(0..)
    }

    pub(crate) fn pop_front(&mut self) -> Option<FullOp> {
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
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum OpAlignment {
    ToMerge((OpSlice, OpSlice)),
    Agreed(Box<FullOp>),
}

/// A struct to help in the tournament sync process. The struct aligns `OpSlice`s so that they can
/// be merged. This includes finding functionally identical operations in each, rectifying the
/// foriegn slice as needed, and providing an interface for exctracting the aligned slices
/// chunk-by-chunk.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct OpAlign {
    // NOTE: Operations are stored in reverse order, so the oldest operations will be at the top of
    // the queue.
    ops: Vec<OpAlignment>,
}

impl OpAlign {
    /// Consumes two op slices that are being used in the sync process. The first slice is the
    /// "known" slice that the other will be compared against. The other slice will be mutated if
    /// they contain operations that are functionality identical.
    pub(crate) fn new(known: OpSlice, foreign: OpSlice) -> Result<Self, SyncError> {
        /* ---- Updates to the foreign slice ---- */
        let player_updates = RefCell::new(Vec::<(PlayerId, PlayerId)>::new());
        let round_updates = RefCell::new(Vec::<(RoundId, RoundId)>::new());

        // The buffer that will be feed foreign operations into if they aren't functionally
        // identical to a known operation
        let mut buffer = OpSlice::new();

        let mut ops = Vec::new();

        let mut iter = foreign.into_iter().map(|mut op| {
            player_updates
                .borrow()
                .iter()
                .for_each(|(old, new)| op.swap_player_ids(*old, *new));
            round_updates
                .borrow()
                .iter()
                .for_each(|(old, new)| op.swap_round_ids(*old, *new));
            op
        });
        let time_update = |old, new| match (old, new) {
            (OpUpdate::None, OpUpdate::None) => {}
            (OpUpdate::PlayerId(old), OpUpdate::PlayerId(new)) => {
                player_updates.borrow_mut().push((old, new));
            }
            (OpUpdate::RoundId(old), OpUpdate::RoundId(new)) => {
                assert_eq!(old.len(), new.len());
                round_updates
                    .borrow_mut()
                    .extend(old.into_iter().zip(new.into_iter()));
            }
            _ => {
                unreachable!(
                    "The inner operations are identical, so their updates should be the same variant."
                );
            }
        };
        for f_op in iter.by_ref() {
            let mut id = None;
            for k_op in known.iter() {
                match k_op.diff(&f_op) {
                    OpDiff::Different => {}
                    OpDiff::Inactive => {
                        todo!("This is an error and should be returned as such.");
                    }
                    OpDiff::Time => {
                        // Functionally identical operations:
                        // Update future operations and add the buffer, the a new slice, and this
                        // operation to the alignment.
                        time_update(k_op.get_update(), f_op.get_update());
                        id = Some(k_op.id);
                        break;
                    }
                    OpDiff::Equal => {
                        id = Some(k_op.id);
                        break;
                    }
                }
            }
            match id {
                None => {
                    buffer.add_op(f_op);
                }
                Some(id) => {
                    // Split the known slice by id
                    let (left, mut right) = known.split_at(id);

                    // Pair the first half of the known slice with the buffer and insert into ops
                    let to_insert = buffer.drain().collect();
                    ops.push(OpAlignment::ToMerge((left, to_insert)));

                    // Pop (from the front) the first op of the second half of the known slice and insert
                    // into ops
                    let k_op = right.pop_front().unwrap();
                    ops.push(OpAlignment::Agreed(Box::new(k_op)));
                }
            }
        }
        // Store the operations in reverse order for easier removal
        Ok(Self {
            ops: ops.drain(0..).rev().collect(),
        })
    }

    /// Removed the next chunk of aligned operations. If `None` is returned, then there are no more
    /// chunks
    pub(crate) fn next(&mut self) -> Option<OpAlignment> {
        self.ops.pop()
    }

    /// Calculates how many chunks of the slices are contained
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// Calculates if any chunks are contained
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    pub fn iter_known(&self) -> impl Iterator<Item = &'_ FullOp> {
        self.ops.iter().rev().flat_map(|al| al.iter_known())
    }

    pub fn iter_foreign(&self) -> impl Iterator<Item = &'_ FullOp> {
        self.ops.iter().rev().flat_map(|al| al.iter_foreign())
    }
}

impl OpAlignment {
    pub(crate) fn iter_known(&self) -> AlignmentIter<'_> {
        match self {
            OpAlignment::Agreed(op) => AlignmentIter::Agreed(Some(op)),
            OpAlignment::ToMerge((known, _)) => AlignmentIter::ToMerge(known.iter()),
        }
    }

    pub(crate) fn iter_foreign(&self) -> AlignmentIter<'_> {
        match self {
            OpAlignment::Agreed(op) => AlignmentIter::Agreed(Some(op)),
            OpAlignment::ToMerge((_, foreign)) => AlignmentIter::ToMerge(foreign.iter()),
        }
    }
}

pub enum AlignmentIter<'a> {
    Agreed(Option<&'a FullOp>),
    ToMerge(vec_deque::Iter<'a, FullOp>),
}

impl<'a> Iterator for AlignmentIter<'a> {
    type Item = &'a FullOp;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            AlignmentIter::Agreed(op) => op.take(),
            AlignmentIter::ToMerge(iter) => iter.next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use squire_tests::{get_seed, spoof_account};

    use super::OpLog;

    #[test]
    fn new_and_init_tourn_test() {
        let owner = spoof_account();
        let seed = get_seed();
        let _log = OpLog::new(owner, seed);
    }
}

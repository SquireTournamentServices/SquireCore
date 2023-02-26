use std::{cell::RefCell, collections::vec_deque};

use serde::{Deserialize, Serialize};
use squire_lib::{operations::OpUpdate, players::PlayerId, rounds::RoundId};

use super::{FullOp, OpDiff, OpSlice, SyncError};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum OpAlignment {
    ToMerge((OpSlice, OpSlice)),
    Agreed(FullOp),
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
                    ops.push(OpAlignment::Agreed(k_op));
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
    pub(crate) fn len(&self) -> usize {
        self.ops.len()
    }

    /// Calculates if any chunks are contained
    pub(crate) fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    pub(crate) fn iter_known(&self) -> impl Iterator<Item = &'_ FullOp> {
        self.ops.iter().rev().map(|al| al.iter_known()).flatten()
    }

    pub(crate) fn iter_foreign(&self) -> impl Iterator<Item = &'_ FullOp> {
        self.ops.iter().rev().map(|al| al.iter_foreign()).flatten()
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

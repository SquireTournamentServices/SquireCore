#[cfg(feature = "server")]
use std::collections::vec_deque::Drain;
use std::collections::vec_deque::{IntoIter, VecDeque};

use serde::{Deserialize, Serialize};

#[cfg(feature = "client")]
use crate::sync::OpSync;
use crate::{
    model::{
        accounts::SquireAccount,
        tournament::{Tournament, TournamentSeed},
    },
    sync::{FullOp, OpId},
};

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

    /// Calculates the length of inner `Vec` of `FullOp`s
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// Calculates if the length of inner `Vec` of `FullOp`s is empty
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Returns an iterator for the log that ignores all elements before the given `OpId`. The
    /// given `OpId` is also ignored. None is returned if the given operation is not found.
    #[cfg(feature = "client")]
    pub(crate) fn iter_passed_op(&self, id: OpId) -> Option<impl Iterator<Item = &FullOp>> {
        let mut iter = self.ops.iter();
        iter.by_ref().find(|op| op.id == id).map(|_| iter)
    }

    /*
    /// Returns an iterator like `iter_passed_op` but also filters operations based on the given
    /// predicate *before* trying to find the operation. None is return if the given operation is
    /// not found.
    pub(crate) fn iter_passed_op_with<F>(&self, id: OpId, mut f: F) -> Option<impl Iterator<Item = &FullOp>>
        where F: FnMut(&FullOp) -> bool,
    {
        let mut iter = self.ops.iter();
        iter.by_ref().filter(|op| f(op)).find(|op| op.id == id).map(|_| iter)
    }
    */

    #[cfg(feature = "client")]
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

    pub(crate) fn get_state_with_slice(&mut self, ops: OpSlice) -> Option<Tournament> {
        let id = ops.first_id()?;
        // TODO: We should actually be able to do better on this check since the log should not
        // have updated since the sync started
        _ = self.ops.iter().rev().find(|op| op.id == id)?;
        let mut tourn = self.init_tourn();
        let mut iter = self.ops.iter().cloned();
        for FullOp { op, salt, .. } in iter.by_ref().take_while(|op| op.id != id) {
            // TODO: This should never error, but if it doesn't, it needs to be logged
            _ = tourn.apply_op(salt, op.clone()).ok()?;
        }
        for FullOp { op, salt, .. } in iter {
            // TODO: This should never error, but if it doesn't, it needs to be logged
            _ = tourn.apply_op(salt, op).ok()?;
        }
        let mut not_found = true;
        self.ops.retain(|op| {
            not_found &= op.id != id;
            not_found
        });
        self.ops.extend(ops);
        Some(tourn)
    }

    /// Creates a slice of this log starting at the given index. `None` is returned if `index` is
    /// out of bounds.
    #[cfg(any(feature = "server", feature = "client"))]
    pub(crate) fn get_slice(&self, id: OpId) -> Option<OpSlice> {
        if self.is_empty() {
            return None;
        }
        let mut not_found = true;
        let mut ops = self
            .ops
            .iter()
            .rev()
            .take_while(|op| std::mem::replace(&mut not_found, op.id != id))
            .cloned()
            .collect::<Vec<_>>();
        if ops.len() < self.len() || ops.last().map(|op| op.id == id).unwrap_or_default() {
            ops.reverse();
            Some(OpSlice { ops: ops.into() })
        } else {
            None
        }
    }

    /// Returns the last operation in the log.
    pub fn last_op(&self) -> Option<FullOp> {
        self.ops.last().cloned()
    }

    /// Returns the id of the last operation in the log.
    pub(crate) fn last_id(&self) -> Option<OpId> {
        self.ops.last().map(|op| op.id)
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

    /// Remove all operations from the backing `VecDeque` while leaving the backing memory
    /// allocated
    #[cfg(feature = "server")]
    pub(crate) fn clear(&mut self) {
        self.ops.clear()
    }

    /// Adds an operation to the end of the OpSlice
    #[cfg(feature = "server")]
    pub(crate) fn add_op(&mut self, op: FullOp) {
        self.ops.push_back(op);
    }

    /// Returns the index of the first stored operation.
    pub(crate) fn first_op(&self) -> Option<FullOp> {
        self.ops.front().cloned()
    }

    /// Returns the index of the first stored operation.
    pub(crate) fn first_id(&self) -> Option<OpId> {
        self.ops.front().map(|o| o.id)
    }

    /// Returns the last operation in the slice.
    pub fn last_op(&self) -> Option<FullOp> {
        self.ops.back().cloned()
    }

    /// Returns the index of the first stored operation.
    #[cfg(feature = "server")]
    pub(crate) fn last_id(&self) -> Option<OpId> {
        self.ops.back().map(|o| o.id)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &FullOp> {
        self.ops.iter()
    }

    #[cfg(feature = "server")]
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

#[cfg(test)]
mod tests {}

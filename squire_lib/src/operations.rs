use std::time::Duration;

use uuid::Uuid;

use crate::{
    player::{Player, PlayerId},
    player_registry::PlayerIdentifier,
    round::{Round, RoundId, RoundResult, RoundStatus},
    round_registry::RoundIdentifier,
    settings::TournamentSetting,
    swiss_pairings::TournamentError,
};

use mtgjson::model::deck::Deck;

use serde::{Deserialize, Serialize};

/// This enum captures all ways in which a tournament can mutate.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TournOp {
    UpdateReg(bool),
    Start(),
    Freeze(),
    Thaw(),
    End(),
    Cancel(),
    CheckIn(PlayerIdentifier),
    RegisterPlayer(String),
    RecordResult(RoundIdentifier, RoundResult),
    ConfirmResult(PlayerIdentifier),
    DropPlayer(PlayerIdentifier),
    AdminDropPlayer(PlayerIdentifier),
    AddDeck(PlayerIdentifier, String, Deck),
    RemoveDeck(PlayerIdentifier, String),
    RemoveRound(RoundIdentifier),
    SetGamerTag(PlayerIdentifier, String),
    ReadyPlayer(PlayerIdentifier),
    UnReadyPlayer(PlayerIdentifier),
    UpdateTournSetting(TournamentSetting),
    GiveBye(PlayerIdentifier),
    CreateRound(Vec<PlayerIdentifier>),
    PairRound(),
    TimeExtension(RoundIdentifier, Duration),
    Cut(usize),
    PruneDecks(),
    PrunePlayers(),
    ImportPlayer(Player),
    ImportRound(Round),
}

impl TournOp {
    pub fn swap_player_ident(self, ident: PlayerIdentifier) -> Self {
        use TournOp::*;
        match self {
            UpdateReg(_)
            | Start()
            | Freeze()
            | Thaw()
            | End()
            | Cancel()
            | RegisterPlayer(_)
            | UpdateTournSetting(_)
            | PairRound()
            | TimeExtension(_, _)
            | Cut(_)
            | PruneDecks()
            | PrunePlayers()
            | RemoveRound(_)
            | ImportPlayer(_)
            | ImportRound(_)
            | RecordResult(_, _)
            | CreateRound(_) => self,
            CheckIn(_) => Self::CheckIn(ident),
            ConfirmResult(_) => Self::ConfirmResult(ident),
            DropPlayer(_) => Self::DropPlayer(ident),
            AdminDropPlayer(_) => Self::AdminDropPlayer(ident),
            AddDeck(_, name, deck) => Self::AddDeck(ident, name, deck),
            RemoveDeck(_, deck) => Self::RemoveDeck(ident, deck),
            SetGamerTag(_, name) => Self::SetGamerTag(ident, name),
            ReadyPlayer(_) => Self::ReadyPlayer(ident),
            UnReadyPlayer(_) => Self::UnReadyPlayer(ident),
            GiveBye(_) => Self::GiveBye(ident),
        }
    }

    pub fn swap_all_player_idents(self, idents: Vec<PlayerIdentifier>) -> Self {
        use TournOp::*;
        match self {
            UpdateReg(_)
            | Start()
            | Freeze()
            | Thaw()
            | End()
            | Cancel()
            | RegisterPlayer(_)
            | UpdateTournSetting(_)
            | PairRound()
            | TimeExtension(_, _)
            | Cut(_)
            | PruneDecks()
            | PrunePlayers()
            | RemoveRound(_)
            | RecordResult(_, _)
            | CheckIn(_)
            | ImportPlayer(_)
            | ImportRound(_)
            | ConfirmResult(_)
            | DropPlayer(_)
            | AdminDropPlayer(_)
            | AddDeck(_, _, _)
            | RemoveDeck(_, _)
            | SetGamerTag(_, _)
            | ReadyPlayer(_)
            | UnReadyPlayer(_)
            | GiveBye(_) => self,
            CreateRound(_) => Self::CreateRound(idents),
        }
    }

    pub fn swap_match_ident(self, ident: RoundIdentifier) -> Self {
        use TournOp::*;
        match self {
            UpdateReg(_)
            | Start()
            | Freeze()
            | Thaw()
            | End()
            | Cancel()
            | CheckIn(_)
            | RegisterPlayer(_)
            | ConfirmResult(_)
            | DropPlayer(_)
            | AdminDropPlayer(_)
            | AddDeck(_, _, _)
            | RemoveDeck(_, _)
            | RemoveRound(_)
            | SetGamerTag(_, _)
            | ReadyPlayer(_)
            | UnReadyPlayer(_)
            | ImportPlayer(_)
            | ImportRound(_)
            | UpdateTournSetting(_)
            | GiveBye(_)
            | CreateRound(_)
            | PairRound()
            | Cut(_)
            | PruneDecks()
            | PrunePlayers() => self,
            TimeExtension(_, dur) => TimeExtension(ident, dur),
            RecordResult(_, res) => RecordResult(ident, res),
        }
    }

    pub fn get_player_ident(&self) -> Option<PlayerIdentifier> {
        use TournOp::*;
        match self {
            UpdateReg(_)
            | Start()
            | Freeze()
            | Thaw()
            | End()
            | Cancel()
            | RegisterPlayer(_)
            | UpdateTournSetting(_)
            | PairRound()
            | TimeExtension(_, _)
            | Cut(_)
            | PruneDecks()
            | ImportPlayer(_)
            | ImportRound(_)
            | PrunePlayers()
            | RemoveRound(_)
            | RecordResult(_, _)
            | CreateRound(_) => None,
            CheckIn(ident)
            | ConfirmResult(ident)
            | DropPlayer(ident)
            | AdminDropPlayer(ident)
            | AddDeck(ident, _, _)
            | RemoveDeck(ident, _)
            | SetGamerTag(ident, _)
            | ReadyPlayer(ident)
            | UnReadyPlayer(ident)
            | GiveBye(ident) => Some(ident.clone()),
        }
    }

    // Used only for CreateRound
    pub fn list_player_ident(&self) -> Option<Vec<PlayerIdentifier>> {
        use TournOp::*;
        match self {
            UpdateReg(_)
            | Start()
            | Freeze()
            | Thaw()
            | End()
            | Cancel()
            | RegisterPlayer(_)
            | UpdateTournSetting(_)
            | PairRound()
            | TimeExtension(_, _)
            | Cut(_)
            | PruneDecks()
            | ImportPlayer(_)
            | ImportRound(_)
            | CheckIn(_)
            | ConfirmResult(_)
            | DropPlayer(_)
            | AdminDropPlayer(_)
            | AddDeck(_, _, _)
            | RemoveDeck(_, _)
            | SetGamerTag(_, _)
            | ReadyPlayer(_)
            | UnReadyPlayer(_)
            | GiveBye(_)
            | PrunePlayers()
            | RemoveRound(_)
            | RecordResult(_, _) => None,
            CreateRound(idents) => Some(idents.clone()),
        }
    }

    pub fn get_match_ident(&self) -> Option<RoundIdentifier> {
        use TournOp::*;
        match self {
            UpdateReg(_)
            | Start()
            | Freeze()
            | Thaw()
            | End()
            | Cancel()
            | CheckIn(_)
            | RegisterPlayer(_)
            | ConfirmResult(_)
            | DropPlayer(_)
            | AdminDropPlayer(_)
            | AddDeck(_, _, _)
            | RemoveDeck(_, _)
            | RemoveRound(_)
            | SetGamerTag(_, _)
            | ImportPlayer(_)
            | ImportRound(_)
            | ReadyPlayer(_)
            | UnReadyPlayer(_)
            | UpdateTournSetting(_)
            | GiveBye(_)
            | CreateRound(_)
            | PairRound()
            | Cut(_)
            | PruneDecks()
            | PrunePlayers() => None,
            TimeExtension(ident, _) | RecordResult(ident, _) => Some(ident.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OpData {
    Nothing,
    RegisterPlayer(PlayerIdentifier),
    ConfirmResult(RoundStatus),
    GiveBye(RoundIdentifier),
    CreateRound(RoundIdentifier),
    Pair(Vec<RoundIdentifier>),
}

pub type OpResult = Result<OpData, TournamentError>;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpId(Uuid);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FullOp {
    pub(crate) op: TournOp,
    pub(crate) id: OpId,
    pub(crate) active: bool,
}

/// An ordered list of all operations applied to a tournament
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpLog {
    pub(crate) ops: Vec<FullOp>,
}

/// An ordered list of some of the operations applied to a tournament
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpSlice {
    pub(crate) ops: Vec<FullOp>,
}

/// A struct to help resolve syncing op logs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpSync {
    pub(crate) ops: OpSlice,
}

/// An enum to help track the progress of the syncing of two op logs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SyncStatus {
    SyncError(OpSync), // Unknown starting operation
    InProgress(Blockage),
    Completed(Synced),
}

/// An enum to that captures the error that might occur when sync op logs.
/// `UnknownOperation` encodes that first operation in an OpSlice is unknown
/// `RollbackFound` encode that a rollback has occured remotely but not locally and returns an
/// OpSlice that contains everything since that rollback. When recieved, this new log should
/// overwrite the local log
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SyncError {
    UnknownOperation(FullOp),
    RollbackFound(OpSlice),
}

/// A struct that marks a completed syncing of op logs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Synced {
    pub(crate) known: OpSlice,
}

/// A struct to help resolve blockages
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Blockage {
    pub(crate) known: OpSlice,
    pub(crate) agreed: OpSlice,
    pub(crate) other: OpSlice,
    pub(crate) problem: (FullOp, FullOp),
}

/// A struct used to communicate a rollback
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rollback {
    pub(crate) ops: OpSlice,
}

impl From<Rollback> for OpSlice {
    fn from(r: Rollback) -> OpSlice {
        r.ops
    }
}

impl Blockage {
    /// Returns the problematic pair of operations.
    pub fn problem(&self) -> (FullOp, FullOp) {
        self.problem.clone()
    }

    /// Resolves the current problem by keeping the given solution and deleting the other, consuming self.
    ///
    /// The `Ok` varient is returned if the given operation is one of the problematic operations.
    /// It contains the rectified logs.
    ///
    /// The `Err` varient is returned if the given operation isn't one of the problematic operations, containing `self`.
    pub fn pick_and_continue(mut self, op: FullOp) -> SyncStatus {
        if op == self.problem.0 {
            self.agreed.add_op(self.problem.0);
        } else if op == self.problem.1 {
            self.agreed.add_op(self.problem.1);
        } else {
            return SyncStatus::InProgress(self);
        }
        match self.known.merge(self.other) {
            Ok(slice) => {
                for op in slice.ops {
                    self.agreed.add_op(op);
                }
                SyncStatus::Completed(Synced { known: self.agreed })
            }
            Err(mut block) => {
                for op in block.agreed.ops {
                    self.agreed.add_op(op);
                }
                block.agreed = self.agreed;
                SyncStatus::InProgress(block)
            }
        }
    }

    /// Resolves the current problem by ordering the problematic solutions, consuming self.
    ///
    /// The `Ok` varient is returned if the given operation is one of the problematic operations.
    /// It contains the rectified logs.
    ///
    /// The `Err` varient is returned if the given operation isn't one of the problematic operations, containing `self`.
    pub fn order_and_continue(mut self, first: FullOp) -> SyncStatus {
        if first == self.problem.0 {
            self.agreed.add_op(self.problem.0);
            self.agreed.add_op(self.problem.1);
        } else if first == self.problem.1 {
            self.agreed.add_op(self.problem.1);
            self.agreed.add_op(self.problem.0);
        } else {
            return SyncStatus::InProgress(self);
        }
        match self.known.merge(self.other) {
            Ok(slice) => {
                for op in slice.ops {
                    self.agreed.add_op(op);
                }
                SyncStatus::Completed(Synced { known: self.agreed })
            }
            Err(mut block) => {
                for op in block.agreed.ops {
                    self.agreed.add_op(op);
                }
                block.agreed = self.agreed;
                SyncStatus::InProgress(block)
            }
        }
    }
}

impl OpLog {
    /// Creates a new log
    pub fn new() -> Self {
        OpLog { ops: Vec::new() }
    }

    pub fn add_op(&mut self, op: FullOp) {
        self.ops.push(op);
    }

    /// Creates a slice of this log starting at the given index. `None` is returned if `index` is
    /// out of bounds.
    pub fn get_slice(&self, id: OpId) -> Option<OpSlice> {
        let mut pass_through = false;
        let ops = self
            .ops
            .iter()
            .filter(|o| {
                pass_through |= o.id == id;
                pass_through
            })
            .cloned()
            .collect();
        Some(OpSlice { ops })
    }

    /// Removes all elements in the log starting at the first index of the given slice. All operations in the slice are then appended to the end of the log.
    pub fn overwrite(&mut self, ops: OpSlice) -> Option<()> {
        let id = ops.start_id()?;
        let index = self.ops.iter().position(|o| o.id == id)?;
        self.ops.truncate(index);
        self.ops.extend(ops.ops.into_iter());
        Some(())
    }

    /// Creates a slice of the current log by starting at the end and moving back. All operations
    /// that cause the closure to return `true` will be dropped and `false` will be kept. An
    /// operation causes `None` to be returned will end the iteration, will not be in the slice,
    /// but kept in the log.
    ///
    /// The primary use case for this is to rollback a round pairing in a Swiss tournament.
    pub fn rollback(&self, mut f: impl FnMut(&FullOp) -> Option<bool>) -> Rollback {
        let l = self.ops.len();
        let ops = self
            .ops
            .iter()
            .rev()
            .map_while(|o| {
                f(o).map(|b| {
                    let mut op = o.clone();
                    op.active = b;
                    op
                })
            })
            .collect();
        Rollback {
            ops: OpSlice { ops },
        }
    }

    /// Attempts to sync the local log with a remote log.
    /// Returns Err if the starting op id of the given log can't be found in this log.
    /// Otherwise, Ok is returned and contains a SyncStatus
    pub fn sync(&mut self, other: OpSync) -> SyncStatus {
        let id = match other.ops.start_id() {
            Some(id) => id,
            None => {
                return SyncStatus::SyncError(other);
            }
        };
        let slice = match self.get_slice(id) {
            Some(s) => s,
            None => {
                return SyncStatus::SyncError(other);
            }
        };
        let merge = slice.merge(other.ops);
        match merge {
            Ok(new_slice) => {
                let index = self.ops.iter().position(|o| o.id == id).unwrap();
                self.ops.truncate(index);
                self.ops.extend(new_slice.ops.iter().cloned());
                SyncStatus::Completed(Synced { known: new_slice })
            }
            Err(block) => SyncStatus::InProgress(block),
        }
    }
}

impl OpSlice {
    /// Creates a new slice
    pub fn new() -> Self {
        OpSlice { ops: Vec::new() }
    }

    pub fn add_op(&mut self, op: FullOp) {
        self.ops.push(op);
    }

    /// Returns the index of the first stored operation.
    pub fn start_id(&self) -> Option<OpId> {
        self.ops.first().map(|o| o.id)
    }

    /// Takes another op slice and attempts to merge it with this log.
    ///
    /// If there are no blockages, the `Ok` varient is returned containing the rectified log and
    /// this log is updated.
    ///
    /// If there is a blockage, the `Err` varient is returned two partial logs, a copy of this log and the
    /// given log. The first operation of  but whose first operations are blocking.
    ///
    /// Promised invarient: If two log can be merged with blockages, they will be meaningfully the
    /// identical; however, identical sequences are not the same. For example, if player A record
    /// their match result and then player B records their result for their (different) match, the
    /// order of these can be swapped without issue.
    ///
    /// The algorithm: For each operation in the given slice, this slice is walked start to finish
    /// until one of the following happens.
    ///     1) An operation identical to the first event in the given log is found. This operation
    ///        is removed from both logs and push onto the new log. We then move to the next
    ///        operation in the given log.
    ///     2) An operation that blocks the first operation in the given log is found. The new log
    ///        is applied, and the current logs are returned to be rectified.
    ///     3) The end of the sliced log is reached and the first operation of the given log is
    ///        removed and push onto the new log.
    /// If there are remaining elements in the sliced log, those are removed and pushed onto the
    /// new log.
    /// The new log is then returned.
    ///
    /// Every operation "knows" what it blocks.
    pub fn merge(mut self, mut other: OpSlice) -> Result<Self, Blockage> {
        todo!()
        /*
        let mut merged = OpSlice::new();
        for (i, (_, other_op)) in other.ops.clone().iter().enumerate() {
            let mut iter = self
                .ops
                .iter()
                .enumerate()
                .take_while(|(i, (_, o))| o != other_op && !o.blocks(other_op));
            let (_, other_op) = other.ops.remove(i);
            if let Some((i, (_, this_op))) = iter.next() {
                let (_, this_op) = self.ops.remove(i);
                if this_op == other_op {
                    merged.add_op(this_op);
                } else {
                    return Err(Blockage {
                        known: self,
                        agreed: merged,
                        other,
                        problem: (this_op, other_op),
                    });
                }
            } else {
                merged.add_op(other_op);
            }
        }
        for (_, o) in self.ops {
            merged.add_op(o);
        }
        Ok(merged)
            */
    }
}

impl Default for OpLog {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for OpSlice {
    fn default() -> Self {
        Self::new()
    }
}

impl FullOp {
    pub fn new(op: TournOp) -> Self {
        Self {
            op,
            id: OpId(Uuid::new_v4()),
            active: true,
        }
    }

    /// Determines if the given operation affects this operation
    pub fn blocks(&self, other: &Self) -> bool {
        self.op.blocks(&other.op)
    }
}

impl TournOp {
    /// Determines if the given operation affects this operation
    pub fn blocks(&self, other: &Self) -> bool {
        todo!()
    }
}

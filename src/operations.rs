/// This enum captures all ways in which a tournament can mutate.
#[derive(Debug, Clone, PartialEq)]
pub enum TournOp {
    UpdateReg,
    Freeze,
    Thaw,
    Start,
    End,
    Cancel,
    CheckIn,
    RegisterPlayer,
    RecordResult,
    ConfirmResult,
    DropPlayer,
    AdminDropPlayer,
    AddDeck,
    RemoveDeck,
    SetGamerTag,
    ReadyPlayer,
    UnReadyPlayer,
    UpdateTournSettings,
    GiveBye,
    CreateRound,
    PairRound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpId(usize);

/// An ordered list of all operations applied to a tournament
pub struct OpLog {
    pub(crate) ops: Vec<TournOp>,
}

/// An ordered list of some of the operations applied to a tournament
pub struct OpSlice {
    pub(crate) ops: Vec<(OpId, TournOp)>,
}

/// A struct used to communicate a rollback
pub struct Rollback {
    ops: OpSlice
}

/// A struct to help resolve blockages
pub struct Blockage {
    known: OpSlice,
    other: OpSlice,
    problem: (TournOp, TournOp),
}

impl From<Rollback> for OpSlice {
    fn from(r: Rollback) -> OpSlice {
        r.ops
    }
}

impl Blockage {
    /// Returns the problematic pair of operations.
    pub fn problem(&self) -> (TournOp, TournOp) {
        self.problem.clone()
    }

    /// Resolves the current problem by keeping the given solution and deleting the other, consuming self.
    ///
    /// The `Ok` varient is returned if the given operation is one of the problematic operations.
    /// It contains the rectified logs.
    ///
    /// The `Err` varient is returned if the given operation isn't one of the problematic operations, containing `self`.
    pub fn pick_resolution(self, op: TournOp) -> Result<(OpLog, OpLog), Self> {
        todo!()
    }

    /// Resolves the current problem by ordering the problematic solutions, consuming self.
    ///
    /// The `Ok` varient is returned if the given operation is one of the problematic operations.
    /// It contains the rectified logs.
    ///
    /// The `Err` varient is returned if the given operation isn't one of the problematic operations, containing `self`.
    pub fn order_resolution(self, first: TournOp) -> Result<(OpLog, OpLog), Self> {
        todo!()
    }
}

impl TournOp {
    /// Determines if the given operation affects this operation
    pub fn blocks(&self, other: &Self) -> bool {
        todo!()
    }
}

impl OpLog {
    /// Creates a new log
    pub fn new() -> Self {
        OpLog {
            ops: Vec::new(),
        }
    }
    
    pub fn add_op(&mut self, op: TournOp) {
        self.ops.push(op);
    }

    /// Creates a slice of this log starting at the given index. `None` is returned if `index` is
    /// out of bounds.
    pub fn get_slice(&self, index: usize) -> Option<OpSlice> {
        if index >= self.ops.len() {
            return None;
        }
        let ops = self
            .ops
            .iter()
            .enumerate()
            .filter(|(i, _)| *i >= index)
            .map(|(i, o)| (OpId(i), o.clone()))
            .collect();
        Some(OpSlice { ops })
    }
    
    /// Removes all elements in the log starting at the first index of the given slice. All
    /// operations in the slice are then appended to the end of the log.
    pub fn overwrite(&mut self, ops: OpSlice) -> Option<()> {
        todo!()
    }
    
    /// Creates a slice of the current log by starting at the end and moving back. All operations
    /// that cause the closure to return `true` will be dropped and `false` will be kept. An
    /// operation causes `None` to be returned will end the iteration, will not be in the slice,
    /// but kept in the log.
    /// 
    /// The primary use case for this is to rollback a round pairing in a Swiss tournament.
    pub fn rollback(&self, f: impl FnMut(TournOp) -> Option<bool>) -> Rollback {
        todo!()
    }

    /// Takes a (partial) op log and attempts to merge it with this log.
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
    /// The algorithm: This log is sliced at the start of the given log. For each operation in the
    /// given log, the sliced log is walked start to finish until one of the following happens.
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
    pub fn merge(&self, other: OpSlice) -> Result<OpSlice, Blockage> {
        todo!()
    }
}

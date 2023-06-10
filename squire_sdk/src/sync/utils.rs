use std::collections::VecDeque;

use crate::sync::{FullOp, OpSlice, OpSync};

impl Default for OpSlice {
    fn default() -> Self {
        Self::new()
    }
}

impl From<VecDeque<FullOp>> for OpSlice {
    fn from(ops: VecDeque<FullOp>) -> Self {
        Self { ops }
    }
}

impl From<OpSync> for OpSlice {
    fn from(s: OpSync) -> OpSlice {
        s.ops
    }
}

/* ---- SyncError Helper Traits ---- */

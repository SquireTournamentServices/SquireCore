use std::ops::Deref;

use serde::{Deserialize, Serialize};
use squire_lib::{
    accounts::SquireAccount,
    operations::{OpData, OpResult, TournOp},
    tournament::{Tournament, TournamentSeed},
};

use crate::sync::ForwardError;

use super::{
    processor::{SyncCompletion, SyncDecision, SyncProcessor},
    FullOp, OpId, OpLog, OpSync, ServerOpLink, SyncError, SyncForwardResp,
};

/// A state manager for the tournament struct
///
/// The manager holds the current tournament and can recreate any meaningful prior state.
///
/// This is the primary synchronization primative between tournaments.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TournamentManager {
    tourn: Tournament,
    log: OpLog,
    /// The last OpId of the last operation after a successful sync
    last_sync: Option<OpId>,
}

impl TournamentManager {
    /// Creates a tournament manager, tournament, and operations log
    pub fn new(owner: SquireAccount, seed: TournamentSeed) -> Self {
        let log = OpLog::new(owner, seed);
        let tourn = log.init_tourn();
        Self {
            tourn,
            log,
            last_sync: None,
        }
    }

    /// Read only accesses to tournaments don't need to be wrapped, so we can freely provide
    /// references to them
    pub fn tourn(&self) -> &Tournament {
        &self.tourn
    }

    /// Takes the manager, removes all unnecessary data for storage, and return the underlying
    /// tournament, consuming the manager in the process.
    pub fn extract(self) -> Tournament {
        self.tourn
    }

    fn apply_op_inner(&mut self, f_op: FullOp) -> OpResult {
        let FullOp { op, salt, .. } = f_op.clone();
        let digest = self.tourn.apply_op(salt, op);
        if digest.is_ok() {
            self.log.ops.push(f_op);
        }
        digest
    }

    fn bulk_apply_ops_inner<I>(&mut self, ops: I) -> OpResult
    where
        I: ExactSizeIterator<Item = FullOp>,
    {
        let mut buffer = self.tourn().clone();
        let mut f_ops = Vec::with_capacity(ops.len());
        for f_op in ops {
            let FullOp { op, salt, .. } = f_op.clone();
            buffer.apply_op(salt, op)?;
            f_ops.push(f_op);
        }
        self.log.ops.extend(f_ops);
        self.tourn = buffer;
        Ok(OpData::Nothing)
    }
}

#[cfg(feature = "server")]
impl TournamentManager {
    /// Consumes an `OpSync`, validates it, and returns the sync processor that will manage the
    /// sync process.
    pub fn init_sync(&mut self, sync: OpSync) -> Result<SyncProcessor, SyncError> {
        SyncProcessor::new(sync, &self.log)
    }

    /// Processes the SyncProcessor and updated the log if it completes without error
    pub fn process_sync(&mut self, mut proc: SyncProcessor) -> ServerOpLink {
        // Check the validity of the incoming processor
        match (proc.last_known(), self.log.last_id()) {
            (Some(id), None) => return SyncError::UnknownOperation(id).into(),
            (None, None) => {}
            (Some(p_id), Some(l_id)) if p_id == l_id => {}
            (Some(_), Some(_)) | (None, Some(_)) => return SyncError::TournUpdated.into(),
        }
        // Bulk apply creates a copy of the tournament state and does not add any operations to the
        // log unless all operations succeed. The `SyncProcessor` will be updated when the
        // `Processing` iterator is dropped.
        match self.bulk_apply_ops_inner(proc.processing()) {
            Ok(_) => proc.finalize().into(),
            Err(_) => proc.into(),
        }
    }

    /// Handles the decision made by the client regarding the sync conflict.
    pub fn handle_decision(&mut self, dec: SyncDecision) -> ServerOpLink {
        match dec {
            SyncDecision::Plucked(proc) => self.process_sync(proc),
            SyncDecision::Purged(comp) => match self.handle_completion(comp.clone()) {
                Ok(()) => comp.into(),
                Err(err) => err.into(),
            },
        }
    }
}

#[cfg(feature = "client")]
impl TournamentManager {
    /// Takes an operation, ensures all idents are their Id variants, stores the operation, applies
    /// it to the tournament, and returns the result.
    pub fn apply_op(&mut self, op: TournOp) -> OpResult {
        self.apply_op_inner(FullOp::new(op.clone()))
    }

    /// Takes an vector of operations and attempts to update the tournament. All operations must
    /// succeed in order for the bulk update the succeed. The update is sandboxed to ensure this.
    pub fn bulk_apply_ops(&mut self, ops: Vec<TournOp>) -> OpResult {
        self.bulk_apply_ops_inner(ops.into_iter().map(FullOp::new))
    }

    /// Method used by clients to create a request for syncing with the remote backend.
    pub fn sync_request(&self) -> OpSync {
        self.log.create_sync_request(self.last_sync)
    }

    /// This method handles a completed sync request returned from the server.
    pub fn handle_completion(&mut self, comp: SyncCompletion) -> Result<(), SyncError> {
        // TODO: Ensure that the tournament has not updated
        match comp {
            // The client's operations were the only operations. There is nothing to update
            SyncCompletion::ForeignOnly(_) => Ok(()),
            SyncCompletion::Mixed(ops) => {
                let Some(id) = ops.first_id() else { return Err(SyncError::EmptySync) };
                let Some(tourn) = self.log.get_state_with_slice(ops) else { return Err(SyncError::UnknownOperation(id)) };
                self.tourn = tourn;
                Ok(())
            }
        }
    }

    /// Handles an sync request that is forwarded from the backend.
    pub fn handle_forwarded_sync(&mut self, sync: OpSync) -> SyncForwardResp {
        println!("Processing forwarded sync request");
        let proc = match SyncProcessor::new(sync, &self.log) {
            Ok(proc) => proc,
            Err(err) => {
                return match err {
                    SyncError::UnknownOperation(_) | SyncError::TournUpdated => {
                        SyncForwardResp::Aborted
                    }
                    SyncError::EmptySync => ForwardError::EmptySync.into(),
                    SyncError::InvalidRequest(err) => err.into(),
                }
            }
        };
        println!("Proc known len: {}", proc.known.len());
        // Ensure the following holds:
        //  Log : anchor op | common ops
        //  Sync: anchor op | common ops | new ops
        //
        //  Note that the sync moves from this to that
        //  Sync           : anchor op  | common ops | new ops
        //  Proc known     : anchor op  | common ops
        //  Proc to_process: common ops | new ops
        if let Some(id) = proc.known.first_id() && proc.known.len() > 1 {
            println!("Verifying that drift did not occur");
            for op in proc.to_process.iter() {
                println!("To-process op id: {}", op.id);
            }
            for op in self.log.ops.iter() {
                println!("Log op id: {}", op.id);
            }
            let mut iter = self.log.ops.iter();
            if iter.by_ref().find(|op| op.id == id).is_none() {
                return ForwardError::EmptySync.into()
            }
            if iter.zip(proc.to_process.iter()).any(|(a, b)| { println!("Checking ops: {}, {}", a.id, b.id); a != b }) {
                return SyncForwardResp::Aborted
            }
        } // TODO: None case? Error?
        // FIXME: Ignore all to_process operations that we have seen
        match self.bulk_apply_ops_inner(proc.to_process.into_iter()) {
            Err(err) => err.into(),
            Ok(_) => SyncForwardResp::Success,
        }
    }

    /// Creates an `OpSync` that will be forwarded to all clients
    pub fn init_sync_forwarding(&self, comp: SyncCompletion) -> OpSync {
        match comp {
            SyncCompletion::ForeignOnly(ops) | SyncCompletion::Mixed(ops) => OpSync {
                owner: self.log.owner.clone(),
                seed: self.log.seed.clone(),
                ops,
            },
        }
    }
}

impl Deref for TournamentManager {
    type Target = Tournament;

    fn deref(&self) -> &Self::Target {
        &self.tourn
    }
}

#[cfg(all(feature = "client", feature = "server"))]
#[cfg(test)]
mod tests {
    use squire_lib::operations::TournOp;
    use squire_tests::{get_seed, spoof_account};

    use crate::sync::{
        processor::SyncCompletion, ServerOpLink, SyncForwardResp, TournamentManager,
    };

    fn spoof_op() -> TournOp {
        TournOp::RegisterPlayer(spoof_account())
    }

    fn init_server_and_clients() -> (TournamentManager, TournamentManager, TournamentManager) {
        // Initialize
        let owner = spoof_account();
        let seed = get_seed();
        let mut c1 = TournamentManager::new(owner.clone(), seed.clone());
        let mut c2 = TournamentManager::new(owner.clone(), seed.clone());
        let mut server = TournamentManager::new(owner, seed);

        // Client one receives an update
        let op = spoof_op();
        c1.apply_op(op.clone()).unwrap();
        assert_eq!(c1.log.len(), 1);
        assert_eq!(c1.log.last_op().unwrap().op, op);
        let sync = c1.sync_request();
        assert_eq!(sync.ops.len(), 1);

        // Client sends update to server
        let proc = server.init_sync(sync).unwrap();
        assert_eq!(proc.known.len(), 0);
        assert_eq!(proc.processed.len(), 0);
        assert_eq!(proc.to_process.len(), 1);
        let link = server.process_sync(proc);
        let ServerOpLink::Completed(comp) = link else { panic!() };
        let SyncCompletion::ForeignOnly(ref ops) = &comp else { panic!() };
        assert_eq!(ops.len(), 1);
        assert_eq!(ops.first_op().unwrap().op, op);
        assert_eq!(server.log.len(), 1);
        assert_eq!(server.log.last_op().unwrap().op, op);

        // Server responds to client one
        c1.handle_completion(comp.clone()).unwrap();
        assert_eq!(c1.log.len(), 1);
        assert_eq!(c1.log.last_op().unwrap().op, op);

        // Server forwards to client two
        let forward = server.init_sync_forwarding(comp);
        let resp = c2.handle_forwarded_sync(forward);
        assert_eq!(resp, SyncForwardResp::Success);
        assert_eq!(c2.log.len(), 1);
        assert_eq!(c2.log.last_op().unwrap().op, op);

        // Done, return the initialized server and clients
        (server, c1, c2)
    }

    // Models what happens during the first sync a full initial sync
    #[test]
    fn initial_sync_test() {
        init_server_and_clients();
    }

    // Models what happens during the second sync of a tournament when client one is ahead of the
    // server
    #[test]
    fn second_sync_test() {
        let (mut server, mut c1, mut c2) = init_server_and_clients();

        // Client one receives an update
        let op = spoof_op();
        println!("{op:?}\n");
        c1.apply_op(op.clone()).unwrap();
        assert_eq!(c1.log.len(), 2);
        assert_eq!(c1.log.last_op().unwrap().op, op);
        let sync = c1.sync_request();
        assert_eq!(sync.ops.len(), 2);

        // Client sends update to server
        let proc = server.init_sync(sync).unwrap();
        assert_eq!(proc.known.len(), 1);
        assert_eq!(proc.processed.len(), 0);
        assert_eq!(proc.to_process.len(), 1);
        let link = server.process_sync(proc);
        println!("{link:?}\n");
        let ServerOpLink::Completed(comp) = link else { panic!() };
        let SyncCompletion::ForeignOnly(ref ops) = &comp else { panic!() };
        assert_eq!(ops.len(), 2);
        assert_eq!(ops.last_op().unwrap().op, op);
        assert_eq!(server.log.len(), 2);
        assert_eq!(server.log.last_op().unwrap().op, op);

        // Server responds to client one
        c1.handle_completion(comp.clone()).unwrap();
        assert_eq!(c1.log.len(), 2);
        assert_eq!(c1.log.last_op().unwrap().op, op);

        // Server forwards to client two
        let forward = server.init_sync_forwarding(comp);
        let resp = c2.handle_forwarded_sync(forward);
        assert_eq!(resp, SyncForwardResp::Success);
        assert_eq!(c2.log.len(), 2);
        assert_eq!(c2.log.last_op().unwrap().op, op);
    }

    // Models what happens during the second sync of a tournament, after client one and the server
    // have drifted but there is no conflict
    #[test]
    fn second_sync_drift_test() {
        let (mut server, mut c1, mut c2) = init_server_and_clients();

        // Client one receives an update
        let client_op = spoof_op();
        println!("{client_op:?}\n");
        c1.apply_op(client_op.clone()).unwrap();
        assert_eq!(c1.log.len(), 2);
        assert_eq!(c1.log.last_op().unwrap().op, client_op);
        let sync = c1.sync_request();
        assert_eq!(sync.ops.len(), 2);

        // Server receives an update before the client syncs
        let server_op = spoof_op();
        println!("{server_op:?}\n");
        server.apply_op(client_op.clone()).unwrap();
        assert_eq!(server.log.len(), 2);
        assert_eq!(server.log.last_op().unwrap().op, client_op);

        // Client sends update to server
        let proc = server.init_sync(sync).unwrap();
        assert_eq!(proc.known.len(), 2);
        assert_eq!(proc.processed.len(), 0);
        assert_eq!(proc.to_process.len(), 1);
        let link = server.process_sync(proc);
        println!("{link:?}\n");
        let ServerOpLink::Completed(comp) = link else { panic!() };
        let SyncCompletion::Mixed(ref ops) = &comp else { panic!() };
        assert_eq!(ops.len(), 3);
        assert_eq!(ops.last_op().unwrap().op, client_op);
        assert_eq!(server.log.len(), 3);
        assert_eq!(server.log.last_op().unwrap().op, client_op);

        // Server responds to client one
        c1.handle_completion(comp.clone()).unwrap();
        assert_eq!(c1.log.len(), 3);
        assert_eq!(c1.log.last_op().unwrap().op, client_op);

        // Server forwards to client two
        let forward = server.init_sync_forwarding(comp);
        let resp = c2.handle_forwarded_sync(forward);
        assert_eq!(resp, SyncForwardResp::Success);
        assert_eq!(c2.log.len(), 3);
        assert_eq!(c2.log.last_op().unwrap().op, client_op);
    }

    // Models what happens during the second sync of a tournament, after client one and the server
    // have drifted and there is a conflict
    #[test]
    fn second_sync_collision_test() {}

    // Models what happens during the second sync of a tournament, after client one and the server
    // have synced but client two and the server have drifted but there is no conflict
    #[test]
    fn ok_forwarded_sync() {
        let (mut server, mut c1, mut c2) = init_server_and_clients();

        // Client one receives an update
        let op = spoof_op();
        println!("{op:?}\n");
        c1.apply_op(op.clone()).unwrap();
        assert_eq!(c1.log.len(), 2);
        assert_eq!(c1.log.last_op().unwrap().op, op);
        let sync = c1.sync_request();
        assert_eq!(sync.ops.len(), 2);

        // Client sends update to server
        let proc = server.init_sync(sync).unwrap();
        assert_eq!(proc.known.len(), 1);
        assert_eq!(proc.processed.len(), 0);
        assert_eq!(proc.to_process.len(), 1);
        let link = server.process_sync(proc);
        println!("{link:?}\n");
        let ServerOpLink::Completed(comp) = link else { panic!() };
        let SyncCompletion::ForeignOnly(ref ops) = &comp else { panic!() };
        assert_eq!(ops.len(), 2);
        assert_eq!(ops.last_op().unwrap().op, op);
        assert_eq!(server.log.len(), 2);
        assert_eq!(server.log.last_op().unwrap().op, op);

        // Server responds to client one
        c1.handle_completion(comp.clone()).unwrap();
        assert_eq!(c1.log.len(), 2);
        assert_eq!(c1.log.last_op().unwrap().op, op);

        // Client two receives an update before the server can forward the sync from client one
        let c2_op = spoof_op();
        println!("{c2_op:?}\n");
        c2.apply_op(c2_op.clone()).unwrap();
        assert_eq!(c2.log.len(), 2);
        assert_eq!(c2.log.last_op().unwrap().op, c2_op);
        let sync = c2.sync_request();
        assert_eq!(sync.ops.len(), 2);

        // Server forwards to client two
        let forward = server.init_sync_forwarding(comp);
        let resp = c2.handle_forwarded_sync(forward);
        assert_eq!(resp, SyncForwardResp::Aborted);
        assert_eq!(c2.log.len(), 2);
        assert_eq!(c2.log.last_op().unwrap().op, c2_op);

        // Client sends update to server
        let sync = c2.sync_request();
        assert_eq!(sync.ops.len(), 2);
        let proc = server.init_sync(sync).unwrap();
        assert_eq!(proc.known.len(), 2);
        assert_eq!(proc.processed.len(), 0);
        assert_eq!(proc.to_process.len(), 1);
        let link = server.process_sync(proc);
        println!("{link:?}\n");
        let ServerOpLink::Completed(comp) = link else { panic!() };
        let SyncCompletion::Mixed(ref ops) = &comp else { panic!() };
        assert_eq!(ops.len(), 3);
        assert_eq!(ops.last_op().unwrap().op, c2_op);
        assert_eq!(server.log.len(), 3);
        assert_eq!(server.log.last_op().unwrap().op, c2_op);

        // Server responds to client two
        c2.handle_completion(comp.clone()).unwrap();
        assert_eq!(c2.log.len(), 3);
        assert_eq!(c2.log.last_op().unwrap().op, c2_op);

        // Server forwards to client one
        let forward = server.init_sync_forwarding(comp);
        let resp = c1.handle_forwarded_sync(forward);
        assert_eq!(resp, SyncForwardResp::Success);
        assert_eq!(c1.log.len(), 3);
        assert_eq!(c1.log.last_op().unwrap().op, c2_op);
    }

    // Models what happens during the second sync of a tournament, after client one and the server
    // have synced but client two and the server have drifted and there is a conflict
    #[test]
    fn conflicted_forwarded_sync() {}

    // Remaining test cases:
    //  A client updates at any point in the syncing process
    //  The server updates at any point in the syncing process
    //  Sanity checks for all error cases captured by SyncError
    //  Multi-stage "random" test where c1 and c2 take turns sending updates to the tournament
    //  (~10 cycles). This tests how the `last_updated` OpId is tracked
}

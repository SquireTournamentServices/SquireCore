use std::ops::Deref;

use serde::{Deserialize, Serialize};
use squire_lib::{
    accounts::SquireAccount,
    tournament::Tournament,
};
use squire_lib::tournament_seed::TournamentSeed;

use super::{OpId, OpLog, processor::SyncCompletion, SyncError};
#[cfg(feature = "server")]
use crate::sync::{processor::SyncDecision, ServerOpLink};
#[cfg(feature = "client")]
use crate::{
    model::operations::TournOp,
    sync::{error::ForwardError, SyncForwardResp},
};
#[cfg(any(feature = "client", feature = "server"))]
use crate::{
    model::operations::{OpData, OpResult},
    sync::{FullOp, OpSync, processor::SyncProcessor},
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

    #[cfg(any(feature = "server", feature = "client"))]
    fn bulk_apply_ops_inner<I>(&mut self, mut ops: I) -> OpResult
    where
        I: ExactSizeIterator<Item = FullOp>,
    {
        let mut buffer = self.tourn().clone();
        let mut f_ops = Vec::with_capacity(ops.len());
        for f_op in ops.by_ref() {
            let FullOp { op, salt, id } = f_op.clone();
            println!("Processing Op {id}");
            buffer.apply_op(salt, op)?;
            f_ops.push(f_op);
        }
        println!("Finished bulk processing operations...");
        self.log.ops.extend(f_ops);
        self.tourn = buffer;
        Ok(OpData::Nothing)
    }

    pub fn seed_and_creator(&self) -> (TournamentSeed, SquireAccount) {
        (self.log.seed.clone(), self.log.owner.clone())
    }

    /// This method handles a completed sync request returned from the server.
    pub fn handle_completion(&mut self, comp: SyncCompletion) -> Result<(), SyncError> {
        let digest = match comp {
            // The client's operations were the only operations. There is nothing to update
            SyncCompletion::ForeignOnly(_) => Ok(()),
            SyncCompletion::Mixed(ops) => {
                let Some(id) = ops.first_id() else {
                    return Err(SyncError::EmptySync);
                };
                let Some(tourn) = self.log.get_state_with_slice(ops) else {
                    return Err(SyncError::UnknownOperation(id));
                };
                self.tourn = tourn;
                Ok(())
            }
        };
        self.last_sync = self.log.last_id();
        digest
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
        let mut iter = proc.processing();
        // Bulk apply creates a copy of the tournament state and does not add any operations to the
        // log unless all operations succeed. The `SyncProcessor` will be updated when the
        // `Processing` iterator is dropped.
        match self.bulk_apply_ops_inner(&mut iter) {
            Ok(_) => {
                iter.conclude();
                proc.finalize().into()
            }
            Err(_) => {
                drop(iter);
                proc.into()
            }
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

#[cfg(feature = "client")]
impl TournamentManager {
    /// Takes an operation, ensures all idents are their Id variants, stores the operation, applies
    /// it to the tournament, and returns the result.
    pub fn apply_op(&mut self, op: TournOp) -> OpResult {
        self.apply_op_inner(FullOp::new(op))
    }

    fn apply_op_inner(&mut self, f_op: FullOp) -> OpResult {
        let FullOp { op, salt, .. } = f_op.clone();
        let digest = self.tourn.apply_op(salt, op);
        if digest.is_ok() {
            self.log.ops.push(f_op);
        }
        digest
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

    /// Handles an sync request that is forwarded from the backend.
    pub fn handle_forwarded_sync(&mut self, sync: OpSync) -> SyncForwardResp {
        let Ok(anchor_id) = sync.first_id() else {
            return SyncForwardResp::Aborted;
        };

        // Ensure the following holds:
        //  Log : known ops | anchor op | common ops
        //  Sync:             anchor op | common ops | new ops
        //
        //  I.e. we don't want the sync to be of the form:
        //  Log : known ops | anchor op | common ops
        //  Sync:             anchor op | new ops    | common ops
        //
        //  Note that the proc divides the sync at the first unknown operation. A well formed proc
        //  should look like this:
        //  Sync           : anchor op | common ops | new ops
        //  Proc known     : anchor op | common ops
        //  Proc to_process: new ops
        //
        //  And a problematic proc will like this:
        //  Sync           : anchor op | common ops | new op | a mix of common and new ops
        //  Proc known     : anchor op | common ops |
        //  Proc to_process: new op    | a mix of common and new ops
        if let Some(iter) = self.log.iter_passed_op(anchor_id) {
            if iter.zip(sync.ops.iter().skip(1)).any(|(a, b)| a != b) {
                return SyncForwardResp::Aborted;
            }
        }
        let proc = match SyncProcessor::new(sync, &self.log) {
            Ok(proc) => proc,
            Err(err) => {
                return match err {
                    SyncError::UnknownOperation(_) | SyncError::TournUpdated => {
                        SyncForwardResp::Aborted
                    }
                    SyncError::EmptySync => ForwardError::EmptySync.into(),
                    SyncError::InvalidRequest(err) => (*err).into(),
                    // TODO: Figure out what to do here... They shouldn't happen
                    SyncError::NotInitialized => todo!(),
                    SyncError::AlreadyInitialized => todo!(),
                    SyncError::AlreadyCompleted => todo!(),
                };
            }
        };

        match self.bulk_apply_ops_inner(proc.to_process.into_iter()) {
            Err(err) => {
                println!("{err:?}");
                err.into()
            }
            Ok(_) => {
                self.last_sync = self.log.last_id();
                SyncForwardResp::Success
            }
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
    use squire_lib::{
        identifiers::AdminId,
        operations::{AdminOp, TournOp},
    };
    use squire_tests::{get_seed, spoof_account};

    use crate::sync::{
        OpSync, processor::SyncCompletion, ServerOpLink, SyncForwardResp, TournamentManager,
    };

    fn reg_op() -> TournOp {
        TournOp::RegisterPlayer(spoof_account(), None)
    }

    fn start_op(admin: AdminId) -> TournOp {
        TournOp::AdminOp(admin, AdminOp::Start)
    }

    fn apply_op(client: &mut TournamentManager, op: TournOp, sync_len: usize) -> OpSync {
        let len = client.log.len();
        client.apply_op(op.clone()).unwrap();
        assert_eq!(client.log.len(), len + 1);
        assert_eq!(client.log.last_op().unwrap().op, op);
        let sync = client.sync_request();
        assert_eq!(sync.len(), sync_len);
        sync
    }

    fn proc_sync(
        server: &mut TournamentManager,
        sync: OpSync,
        proc_len: [usize; 3],
        op: &TournOp,
    ) -> ServerOpLink {
        let log_len = server.log.len();
        let proc = server.init_sync(sync).unwrap();
        assert_eq!(proc.known.len(), proc_len[0]);
        assert_eq!(proc.processed.len(), proc_len[1]);
        assert_eq!(proc.to_process.len(), proc_len[2]);
        let link = server.process_sync(proc);
        let ServerOpLink::Completed(comp) = link.clone() else {
            panic!()
        };
        let SyncCompletion::ForeignOnly(ref ops) = &comp else {
            panic!()
        };
        assert_eq!(ops.len(), proc_len[0] + proc_len[2]);
        assert_eq!(&ops.last_op().unwrap().op, op);
        assert_eq!(server.log.len(), log_len + proc_len[2]);
        assert_eq!(&server.log.last_op().unwrap().op, op);
        link
    }

    fn init_server_and_clients() -> (TournamentManager, TournamentManager, TournamentManager) {
        // Initialize
        let owner = spoof_account();
        let seed = get_seed();
        let mut c1 = TournamentManager::new(owner.clone(), seed.clone());
        let mut c2 = TournamentManager::new(owner.clone(), seed.clone());
        let mut server = TournamentManager::new(owner, seed);
        assert!(c1.last_sync.is_none());
        assert!(c2.last_sync.is_none());
        assert!(server.last_sync.is_none());

        // Client one receives an update
        let op = reg_op();
        let sync = apply_op(&mut c1, op.clone(), 1);

        // Client sends update to server
        let link = proc_sync(&mut server, sync, [0, 0, 1], &op);
        let ServerOpLink::Completed(comp) = link else {
            panic!()
        };

        // Server responds to client one
        c1.handle_completion(comp.clone()).unwrap();
        assert_eq!(c1.log.len(), 1);
        assert_eq!(c1.log.last_op().unwrap().op, op);
        assert_eq!(c1.last_sync, c1.log.last_id());

        // Server forwards to client two
        let forward = server.init_sync_forwarding(comp);
        let resp = c2.handle_forwarded_sync(forward);
        assert_eq!(resp, SyncForwardResp::Success);
        assert_eq!(c2.log.len(), 1);
        assert_eq!(c2.log.last_op().unwrap().op, op);
        assert_eq!(c2.last_sync, c2.log.last_id());

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
        let op = reg_op();
        apply_op(&mut c1, op.clone(), 2);
        let sync = c1.sync_request();

        // Client sends update to server
        let link = proc_sync(&mut server, sync, [1, 0, 1], &op);
        let ServerOpLink::Completed(comp) = link else {
            panic!()
        };

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

    #[test]
    fn multiple_sync_test() {
        let (mut server, mut c1, mut c2) = init_server_and_clients();

        for i in 0..100 {
            println!("Starting sync cycle #{i}");
            let c1_op_len = c1.log.len();
            let c2_op_len = c2.log.len();

            // Client one receives an update
            let op = reg_op();
            apply_op(&mut c1, op.clone(), 2);
            let sync = c1.sync_request();

            // Client sends update to server
            let link = proc_sync(&mut server, sync, [1, 0, 1], &op);
            let ServerOpLink::Completed(comp) = link else {
                panic!()
            };

            // Server responds to client one
            c1.handle_completion(comp.clone()).unwrap();
            assert_eq!(c1.log.len(), c1_op_len + 1);
            assert_eq!(c1.log.last_op().unwrap().op, op);

            // Server forwards to client two
            let forward = server.init_sync_forwarding(comp);
            let resp = c2.handle_forwarded_sync(forward);
            assert_eq!(resp, SyncForwardResp::Success);
            assert_eq!(c2.log.len(), c2_op_len + 1);
            assert_eq!(c2.log.last_op().unwrap().op, op);
        }
    }

    // Models what happens during the second sync of a tournament, after client one and the server
    // have drifted but there is no conflict
    #[test]
    fn second_sync_drift_test() {
        let (mut server, mut c1, mut c2) = init_server_and_clients();

        // Client one receives an update
        let client_op = reg_op();
        let sync = apply_op(&mut c1, client_op.clone(), 2);

        // Server receives an update before the client syncs
        let server_op = reg_op();
        println!("{server_op:?}\n");
        server.apply_op(server_op.clone()).unwrap();
        assert_eq!(server.log.len(), 2);
        assert_eq!(server.log.last_op().unwrap().op, server_op);

        // Client sends update to server
        let proc = server.init_sync(sync).unwrap();
        assert_eq!(proc.known.len(), 2);
        assert_eq!(proc.processed.len(), 0);
        assert_eq!(proc.to_process.len(), 1);
        let link = server.process_sync(proc);
        println!("{link:?}\n");
        let ServerOpLink::Completed(comp) = link else {
            panic!()
        };
        let SyncCompletion::Mixed(ref ops) = &comp else {
            panic!()
        };
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
    fn second_sync_collision_test() {
        let (mut server, mut c1, mut c2) = init_server_and_clients();
        let admin = *server.tourn.admins.iter().next().unwrap().0;
        let _c1_op_len = c1.log.len();
        let _c2_op_len = c2.log.len();
        let server_op_len = server.log.len();

        // Client one receives an update
        let c1_op = reg_op();
        println!("{c1_op:?}");
        c1.apply_op(c1_op.clone()).unwrap();
        assert_eq!(c1.log.len(), 2);
        assert_eq!(c1.log.last_op().unwrap().op, c1_op);
        let c1_sync = c1.sync_request();
        assert_eq!(c1_sync.ops.len(), 2);

        // Client two receives an update
        let c2_op = start_op(admin);
        println!("{c2_op:?}");
        c2.apply_op(c2_op.clone()).unwrap();
        assert_eq!(c2.log.len(), 2);
        assert_eq!(c2.log.last_op().unwrap().op, c2_op);
        let c2_sync = c2.sync_request();
        assert_eq!(c2_sync.ops.len(), 2);

        // Server receives C2's update before C1's
        println!("Init sync between C2 and server...");
        let proc = server.init_sync(c2_sync).unwrap();
        assert_eq!(proc.known.len(), 1);
        assert_eq!(proc.processed.len(), 0);
        assert_eq!(proc.to_process.len(), 1);
        let link = server.process_sync(proc);
        println!("Server processed sync init...");
        println!("{link:?}\n");
        let ServerOpLink::Completed(comp) = link else {
            panic!()
        };
        let SyncCompletion::ForeignOnly(ref ops) = &comp else {
            panic!()
        };
        assert_eq!(ops.len(), 2);
        assert_eq!(ops.last_op().unwrap().op, c2_op);
        assert_eq!(server.log.len(), server_op_len + 1);
        assert_eq!(server.log.last_op().unwrap().op, c2_op);

        // Server responds to client two
        c2.handle_completion(comp.clone()).unwrap();
        assert_eq!(c2.log.len(), 2);
        assert_eq!(c2.log.last_op().unwrap().op, c2_op);
        assert_eq!(c2.last_sync, c2.log.last_id());

        // Server forwards to client one
        let forward = server.init_sync_forwarding(comp);
        let resp = c1.handle_forwarded_sync(forward);
        // Aborted because the log and sync look like this:
        // C1's log: init op | C1 op
        // Sync    : init op | C2 op
        assert_eq!(resp, SyncForwardResp::Aborted);
        assert_eq!(c1.log.len(), 2);
        assert_eq!(c1.log.last_op().unwrap().op, c1_op);

        // Client one sends update to server
        let proc = server.init_sync(c1_sync).unwrap();
        assert_eq!(proc.known.len(), 2);
        assert_eq!(proc.processed.len(), 0);
        assert_eq!(proc.to_process.len(), 1);
        let link = server.process_sync(proc);
        println!("{link:?}\n");
        let ServerOpLink::Conflict(conflict) = link else {
            panic!()
        };
        assert_eq!(conflict.known.len(), 2);
        assert_eq!(conflict.processed.len(), 0);
        assert_eq!(conflict.to_process.len(), 1);
        assert_eq!(server.log.len(), 2);
        assert_eq!(server.log.last_op().unwrap().op, c2_op);

        // Server resolves conflict via purging and responses
        let decision = conflict.purge();
        let link = server.handle_decision(decision);
        let ServerOpLink::Completed(comp) = link else {
            panic!()
        };
        assert_eq!(comp.len(), 2);
        assert_eq!(server.log.len(), 2);
        assert_eq!(server.log.last_op().unwrap().op, c2_op);

        // Client one receives the completed sync
        c1.handle_completion(comp.clone()).unwrap();
        assert_eq!(c1.log.len(), 2);
        assert_eq!(c1.log.last_op().unwrap().op, c2_op);
        assert_eq!(c1.last_sync, c1.log.last_id());

        // Server forwards to client two (effectively a noop)
        let forward = server.init_sync_forwarding(comp);
        assert_eq!(forward.ops.len(), 2);
        let resp = c2.handle_forwarded_sync(forward);
        assert_eq!(resp, SyncForwardResp::Success);
        assert_eq!(c2.log.len(), 2);
        assert_eq!(c2.log.last_op().unwrap().op, c2_op);
        assert_eq!(c2.last_sync, c2.log.last_id());
    }

    // Models what happens during the second sync of a tournament, after client one and the server
    // have synced but client two and the server have drifted but there is no conflict
    #[test]
    fn ok_forwarded_sync() {
        let (mut server, mut c1, mut c2) = init_server_and_clients();

        // Client one receives an update
        let op = reg_op();
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
        let ServerOpLink::Completed(comp) = link else {
            panic!()
        };
        let SyncCompletion::ForeignOnly(ref ops) = &comp else {
            panic!()
        };
        assert_eq!(ops.len(), 2);
        assert_eq!(ops.last_op().unwrap().op, op);
        assert_eq!(server.log.len(), 2);
        assert_eq!(server.log.last_op().unwrap().op, op);

        // Server responds to client one
        c1.handle_completion(comp.clone()).unwrap();
        assert_eq!(c1.log.len(), 2);
        assert_eq!(c1.log.last_op().unwrap().op, op);

        // Client two receives an update before the server can forward the sync from client one
        let c2_op = reg_op();
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
        let ServerOpLink::Completed(comp) = link else {
            panic!()
        };
        let SyncCompletion::Mixed(ref ops) = &comp else {
            panic!()
        };
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

    // TODO: I think this is covered by second sync collision test
    // Models what happens during the second sync of a tournament, after client one and the server
    // have synced but client two and the server have drifted and there is a conflict
    // #[test]
    // fn conflicted_forwarded_sync() {}

    // Remaining test cases:
    //   - A client updates at any point in the syncing process
    //   - The server updates at any point in the syncing process
    //   - Sanity checks for all error cases captured by SyncError
    //   - Multi-stage "random" test where c1 and c2 take turns sending updates to the tournament
    //  (~100 cycles). This tests how the `last_updated` OpId is tracked
    //   - C1 sends a sync request to server. Sync completes but the completion is not sent to the
    //   C1. C1 sends a new sync request. Sync request should automatically complete. Test both
    //   ForeignOnly and Mixed completion and with(out) the tournament get other updates (four
    //   cases)
}

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
pub use squire_lib::{
    error::TournamentError,
    identifiers::{TournamentId, TournamentIdentifier},
    operations::{OpResult, TournOp},
    scoring::Standings,
    standard_scoring::StandardScore,
    tournament::{Tournament, TournamentPreset},
};
use squire_lib::{
    identifiers::OpId,
    operations::{OpSlice, OpSync, Rollback, RollbackError, SyncStatus},
};

use crate::response::SquireResponse;

/// The response type used by the `tournaments/<id>/get` SC API. The option encodes that the
/// requested tournament might not be found.
pub type GetTournamentResponse = SquireResponse<Option<Tournament>>;

/// The response type used by the `tournaments/all` SC API. The inner data is a map between
/// tournament id and tournament objects.
pub type AllTournamentsResponse = SquireResponse<HashMap<TournamentId, Tournament>>;

#[derive(Debug, Serialize, Deserialize)]
/// The request type taking by the `tournaments/create` SC API. The fields contain all the data
/// required to create a tournament.
pub struct CreateTournamentRequest {
    /// The name of the new tournament
    pub name: String,
    /// The preset of the new tournament
    pub preset: TournamentPreset,
    /// The format of the new tournament
    pub format: String,
}

/// The response type used by the `tournaments/all` SC API. The inner data is the newly created
/// tournament object.
pub type CreateTournamentResponse = SquireResponse<Tournament>;

/// The response type used by the `tournaments/<id>/standings` SC API. The option encodes that the
/// requested tournament might not be found. The inner data is the sorted standings from the
/// tournament.
pub type StandingsResponse = SquireResponse<Option<Standings<StandardScore>>>;

/// The response type used by the `tournaments/<t_id>/manage/ops/slice/<o_id>` SC API. The nested options
/// encode that the requested tournament and operation might not be found. The inner data is an
/// ordered list of operations that start with the requested operation.
pub type OpSliceResponse = SquireResponse<Option<Option<OpSlice>>>;

#[derive(Debug, Serialize, Deserialize)]
/// The request type taking by the `tournaments/<t_id>/manage/refresh` SC API. The fields contain
/// all the data to test if the client and server are synced. If in sync, the client and server
/// shold both have `length` many operations, a last operation with `op_id` as it id and be
/// `active`. If any of these are mismatches, they are somewhat out of sync.
pub struct RefreshRequest {
    /// The id of the last known operation on the client's side
    pub op_id: OpId,
    /// The status of the last known operation on the client's side
    pub active: bool,
    /// The number of operations the client is aware of
    pub length: usize,
}

#[derive(Debug, Serialize, Deserialize)]
/// The enum encodes all the outcomes of a refresh request
pub enum RefreshResult {
    /// Everything is up do date. Client and server are in sync
    UpToDate,
    /// The client has new operations and returns them.
    NewOps(OpSync),
    /// A rollback has occurred and returns the list of rolled back operations
    Rollback(Rollback),
    /// Some kind of error has occured
    Error(RefreshError),
}

#[derive(Debug, Serialize, Deserialize)]
/// The enum encodes all errors that might outcome during a refresh request
pub enum RefreshError {
    /// The requested operations was not found. This likely means there was a sync that deleted
    /// that operation
    OpNotFound(OpId),
    /// The requested operation was found but was not where it was expected. Thsi likely is because
    /// a sync reordered these operations
    OutOfOrder {
        /// The operation id that was part of the request
        expected: OpId,
        /// The operation id that was found in its place
        found: OpId,
    },
}

/// The response type used by the `tournaments/<t_id>/manage/refresh` SC API. The option encodes
/// that the requested tournament might not be found. The inner data is a variant of the
/// `RefreshResult` enum.
pub type RefreshResponse = SquireResponse<Option<RefreshResult>>;

#[derive(Debug, Serialize, Deserialize)]
/// The request type taking by the `tournaments/<t_id>/manage/sync` SC API.
pub struct SyncRequest {
    /// The `OpSync` needed to attempt to perform the sync.
    pub sync: OpSync,
}

/// The response type used by the `tournaments/<t_id>/manage/sync` SC API.
/// The option encodes that the requested tournament might not be found. The inner data is the
/// result of attempting to apply the sync.
pub type SyncResponse = SquireResponse<Option<SyncStatus>>;

#[derive(Debug, Serialize, Deserialize)]
/// The request type taking by the `tournaments/<t_id>/manage/rollback` SC API.
pub struct RollbackRequest {
    /// The `Rollback` needed to attempt to perform the rollback.
    pub rollback: Rollback,
}

/// The response type used by the `tournaments/<t_id>/manage/rollback` SC API.
/// The option encodes that the requested tournament might not be found. The inner data is the
/// result of attempting to apply the rollback.
pub type RollbackResponse = SquireResponse<Option<Result<(), RollbackError>>>;

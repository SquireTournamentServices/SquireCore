use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use squire_lib::operations::{OpSlice, OpSync, Rollback, RollbackError, SyncStatus};
pub use squire_lib::{
    error::TournamentError,
    identifiers::{TournamentId, TournamentIdentifier},
    operations::{OpResult, TournOp},
    scoring::Standings,
    standard_scoring::StandardScore,
    tournament::{Tournament, TournamentPreset},
};

use crate::response::SquireResponse;

pub type TournamentGetResponse = SquireResponse<Option<Tournament>>;

pub type GetAllResponse = SquireResponse<HashMap<TournamentId, Tournament>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct TournamentCreateRequest {
    pub name: String,
    pub preset: TournamentPreset,
    pub format: String,
}

pub type CreateResponse = SquireResponse<Tournament>;

pub type StandingsResponse = SquireResponse<Option<Standings<StandardScore>>>;

pub type OpSliceResponse = SquireResponse<Option<OpSlice>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRequest {
    pub sync: OpSync,
}

pub type SyncResponse = SquireResponse<Option<SyncStatus>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncImportRequest {
    pub sync: OpSync,
}

pub type SyncImportResponse = SquireResponse<Option<SyncStatus>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct RollbackRequest {
    pub sync: Rollback,
}

pub type RollbackResponse = SquireResponse<Option<Result<(), RollbackError>>>;

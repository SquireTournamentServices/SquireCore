use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use squire_lib::operations::{OpSlice, OpSync, Rollback, SyncStatus, Synced};
pub use squire_lib::{
    error::TournamentError,
    operations::{OpResult, TournOp},
    scoring::Standings,
    standard_scoring::StandardScore,
    tournament::{Tournament, TournamentId, TournamentIdentifier, TournamentPreset},
};

use crate::response::SquireResponse;

#[derive(Debug, Serialize, Deserialize)]
pub struct TournamentGetRequest {
    pub ident: TournamentIdentifier,
}

pub type GetResponse = SquireResponse<Option<Tournament>>;

pub type GetAllResponse = SquireResponse<HashMap<TournamentId, Tournament>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct TournamentCreateRequest {
    pub name: String,
    pub preset: TournamentPreset,
    pub format: String,
}

pub type CreateResponse = SquireResponse<Tournament>;

#[derive(Debug, Serialize, Deserialize)]
pub struct StandingsRequest {
    pub ident: TournamentIdentifier,
}

pub type StandingsResponse = SquireResponse<Option<Standings<StandardScore>>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListOpsRequest {
    pub ident: TournamentIdentifier,
}

pub type ListOpsResponse = SquireResponse<Option<OpSlice>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRequest {
    pub ident: TournamentIdentifier,
    pub sync: OpSync,
}

pub type SyncResponse = SquireResponse<Option<SyncStatus>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncImportRequest {
    pub ident: TournamentIdentifier,
    pub sync: Synced,
}

pub type SyncImportResponse = SquireResponse<Option<Result<Synced, SyncStatus>>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct RollbackRequest {
    pub ident: TournamentIdentifier,
    pub sync: Rollback,
}

pub type RollbackResponse = SquireResponse<Option<SyncStatus>>;

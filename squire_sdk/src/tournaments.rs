use std::collections::HashMap;

use serde::{Deserialize, Serialize};
pub use squire_lib::{
    error::TournamentError, operations::OpResult, scoring::Standings,
    standard_scoring::StandardScore, tournament::Tournament, tournament::TournamentId,
};
use squire_lib::{
    operations::TournOp,
    tournament::{TournamentIdentifier, TournamentPreset},
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
pub struct ApplyOpRequest {
    pub ident: TournamentIdentifier,
    pub operation: TournOp,
}

pub type ApplyOpResponse = SquireResponse<Option<OpResult>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct StandingsRequest {
    pub ident: TournamentIdentifier,
}

pub type StandingsResponse = SquireResponse<Option<Standings<StandardScore>>>;

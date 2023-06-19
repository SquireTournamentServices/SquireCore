use serde::{Deserialize, Serialize};

use crate::response::SquireResponse;
pub use crate::{
    model::{
        error::TournamentError,
        identifiers::{TournamentId, TournamentIdentifier},
        operations::{OpResult, TournOp},
        scoring::{StandardScore, Standings},
        tournament::{Tournament, TournamentPreset},
    },
    sync::{OpId, OpSlice, OpSync, TournamentManager},
};

/// The response type used by the `tournaments/<id>/get` SC API. The option encodes that the
/// requested tournament might not be found.
pub type GetTournamentResponse = SquireResponse<Option<TournamentManager>>;

/// The response type used by the `tournaments/all` SC API. The option encodes that the
/// requested tournament might not be found.
pub type GetAllTournamentsResponse = SquireResponse<Vec<TournamentManager>>;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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
pub type CreateTournamentResponse = SquireResponse<TournamentManager>;

use serde::{Deserialize, Serialize};
use squire_lib::tournament::TournamentStatus;

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

/// Information useful for understanding the tournament at a glance, as well as for performing a
/// query to find out more about it.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct TournamentSummary {
    /// The unique identifier for the tournament -- this can be used to query the backend for more
    /// information about a tournament (using [server::ServerState::get_tourn])
    pub id: TournamentId,
    /// The display name of the tournament
    pub name: String,
    /// The status of the tournament
    pub status: TournamentStatus,
}

impl From<&Tournament> for TournamentSummary {
    fn from(value: &Tournament) -> Self {
        Self {
            id: value.id,
            name: value.name.clone(),
            status: value.status,
        }
    }
}

impl From<&TournamentManager> for TournamentSummary {
    fn from(value: &TournamentManager) -> Self {
        Self::from(value.tourn())
    }
}

fn default_page_size() -> usize {
    20
}

/// The query parameter used by the `tournaments/list/<page>[?page_size=number]` SC API. This query
/// parameter is not necessary, and defaults to 20 if not specified. The vector does not necessarily
/// contain as many elements as the page size, *even when you haven't reached the end of the
/// complete list of tournaments*.
#[derive(Deserialize, Debug)]
pub struct ListPageSize {
    #[serde(default = "default_page_size")]
    pub page_size: usize,
}

/// The response type used by the `tournaments/list/<page>[?page_size=number]` SC API. The vector
/// returned contains a list of tournament summaries, which each contain an ID which can be used to
/// query more about the tournament. The vector does not necessarily contain as many elements as the
/// page size, *even when you haven't reached the end of the complete list of tournaments*.
pub type ListTournamentsResponse = SquireResponse<Vec<TournamentSummary>>;

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

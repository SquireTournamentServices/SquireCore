use axum::{extract::State, routing::get, Router};

use squire_sdk::{
    api::GET_ALL_PAST_TOURNAMENTS_ENDPOINT, server::state::ServerState,
    tournaments::GetAllPastTournamentsResponse,
};

use crate::state::AppState;

pub(crate) fn get_routes() -> Router<AppState> {
    Router::new().route(
        GET_ALL_PAST_TOURNAMENTS_ENDPOINT.as_str(),
        get(get_all_past_tournaments),
    )
}

pub async fn get_all_past_tournaments(
    State(state): State<AppState>,
) -> GetAllPastTournamentsResponse {
    GetAllPastTournamentsResponse::new(
        state
            .query_all_past_tournaments(|t| (t.id, t.clone()))
            .await,
    )
}

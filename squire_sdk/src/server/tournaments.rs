use std::sync::Arc;

use async_session::SessionStore;
use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    handler::Handler,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use squire_lib::tournament::TournamentId;

use super::gathering::{self, handle_new_onlooker};
use crate::{
    api::{GET_TOURNAMENT_ENDPOINT, LIST_TOURNAMENTS_ENDPOINT, SUBSCRIBE_ENDPOINT},
    server::{state::ServerState, User},
    tournaments::*,
    utils::Url,
};

pub fn get_routes_and_init<S: ServerState>(state: S) -> Router<S> {
    gathering::init_gathering_hall(state);
    get_routes()
}

pub fn get_routes<S: ServerState>() -> Router<S> {
    Router::new()
        .route(
            LIST_TOURNAMENTS_ENDPOINT.as_str(),
            get(get_tournament_list::<S>),
        )
        .route(GET_TOURNAMENT_ENDPOINT.as_str(), get(get_tournament::<S>))
        .route(SUBSCRIBE_ENDPOINT.as_str(), get(join_gathering::<S>))
}

/// Returns a list of [TournamentSummary], which can be used to see information about a collection
/// of tournaments at a glance and query more information about a particular tournament using the
/// `id` field in [get_tournament].
///
/// This api can be accessed via `/api/v1/tournaments/list/<page>[?page_size=number]`. For example,
/// `/api/v1/tournaments/list/0` will give back at most 20 tournament summaries, starting from the
/// latest registered tournament possible. The returned list won't always contain 20 summaries (for
/// example, if the backend fails to fetch some elements individually or ran out of tournaments to
/// return from), but it almost always will. In the same way, accessing
/// `/api/v1/tournaments/list/0?page_size=10` will give back at most 10 summaries, starting from the
/// latest registered tournament possible.
pub async fn get_tournament_list<S>(
    State(state): State<S>,
    Path(page): Path<usize>,
    Query(ListPageSize { page_size }): Query<ListPageSize>,
) -> ListTournamentsResponse
where
    S: ServerState,
{
    dbg!(page_size);
    let offset = page * page_size;
    ListTournamentsResponse::new(
        state
            .get_tourn_summaries(offset..(offset + page_size))
            .await,
    )
}

pub async fn get_tournament<S>(
    State(state): State<S>,
    Path(id): Path<TournamentId>,
) -> GetTournamentResponse
where
    S: ServerState,
{
    GetTournamentResponse::new(state.get_tourn(id).await)
}

/// Adds a user to the gathering via a websocket
pub async fn join_gathering<S>(
    user: User,
    ws: WebSocketUpgrade,
    Path(id): Path<TournamentId>,
) -> Response {
    println!("Got Websocket request...");
    ws.on_upgrade(move |ws| {
        println!("Websocket request got upgraded...");
        handle_new_onlooker(id, user, ws)
    })
}

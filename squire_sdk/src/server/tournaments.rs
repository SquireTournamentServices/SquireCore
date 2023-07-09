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
            .get_tourn_summaries(offset..=(offset + page_size - 1))
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

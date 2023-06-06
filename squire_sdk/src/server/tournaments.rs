use std::sync::Arc;

use async_session::SessionStore;
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    handler::Handler,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};

use crate::{
    api::{GET_TOURNAMENT_ENDPOINT, SUBSCRIBE_ENDPOINT},
    server::{state::ServerState, User},
    tournaments::*,
    utils::Url,
};

use super::gathering::{self, handle_new_onlooker};

pub fn get_routes_and_init<S: ServerState>(state: S) -> Router<S> {
    gathering::init_gathering_hall(state);
    get_routes()
}

pub fn get_routes<S: ServerState>() -> Router<S> {
    Router::new()
        .route(GET_TOURNAMENT_ENDPOINT.as_str(), get(get_tournament::<S>))
        .route(SUBSCRIBE_ENDPOINT.as_str(), get(join_gathering::<S>))
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

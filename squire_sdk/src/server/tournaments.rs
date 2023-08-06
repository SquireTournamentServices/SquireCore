use axum::{
    body::Body,
    extract::{Path, Query, State, WebSocketUpgrade},
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use squire_lib::tournament::TournamentId;

use super::{
    gathering::{self, handle_new_onlooker},
    SquireRouter, session::UserSession,
};
use crate::{api::*, server::state::ServerState, sync::TournamentManager};

pub fn get_routes_and_init<S: ServerState>(state: S) -> SquireRouter<S, Body> {
    gathering::init_gathering_hall(state);
    get_routes()
}

pub fn get_routes<S: ServerState>() -> SquireRouter<S> {
    SquireRouter::new()
        //.add_route("/", post(import_tournament::<S>))
        .add_route::<1, GET, ListTournaments, _, _>(get_tournament_list::<S>)
        .add_route::<1, GET, GetTournament, _, _>(get_tournament::<S>)
        .add_route::<1, GET, Subscribe, _, _>(join_gathering::<S>)
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

pub async fn import_tournament<S>(
    State(state): State<S>,
    _user: UserSession,
    Json(tourn): Json<TournamentManager>,
) -> impl IntoResponse
where
    S: ServerState,
{
    match state.get_tourn(tourn.id).await {
        Some(_) => StatusCode::BAD_REQUEST,
        None => {
            let _ = state.persist_tourn(&tourn).await;
            StatusCode::OK
        }
    }
}

/// Adds a user to the gathering via a websocket
pub async fn join_gathering<S>(
    user: UserSession,
    ws: WebSocketUpgrade,
    Path(id): Path<TournamentId>,
) -> Response {
    ws.on_upgrade(move |ws| handle_new_onlooker(id, user, ws))
}

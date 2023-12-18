use axum::{routing::get, Router};
use mongodb::Database;
use squire_sdk::{api::*, server};
use tower_http::cors::CorsLayer;

#[cfg(test)]
mod tests;

#[cfg(not(debug_assertions))]
mod assets;

mod accounts;
mod session;
mod state;

use accounts::*;
use session::*;
use state::{AppState, AppStateBuilder};

pub fn create_router(state: AppState) -> Router {
    let router = server::create_router::<AppState>()
        .add_route::<0, POST, RegForm, _, _>(create_account)
        .add_route::<0, GET, AccountCrud, _, _>(get_account)
        .add_route::<0, DELETE, AccountCrud, _, _>(delete_account)
        .add_route::<0, POST, Login, _, _>(login)
        .add_route::<0, POST, GuestSession, _, _>(guest)
        .add_route::<0, POST, Reauth, _, _>(reauth)
        .add_route::<0, DELETE, Terminate, _, _>(terminate)
        .add_route::<0, GET, GetSessionStatus, _, _>(status)
        .into_router();

    #[cfg(not(debug_assertions))]
    let router = assets::inject_ui(router);

    router.layer(CorsLayer::permissive()).with_state(state)
}

#[shuttle_runtime::main]
async fn axum(#[shuttle_shared_db::MongoDb] db_conn: Database) -> shuttle_axum::ShuttleAxum {
    let app_state = AppStateBuilder::with_db(db_conn).build();
    Ok(create_router(app_state).into())
}

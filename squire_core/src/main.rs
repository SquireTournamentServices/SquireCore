use axum::{routing::get, Router};
use mongodb::Database;
use squire_sdk::server;
use state::{AppState, AppStateBuilder};

#[cfg(test)]
mod tests;

mod assets;
mod state;

pub fn create_router(state: AppState) -> Router {
    server::create_router::<AppState>(state.clone())
        .into_router()
        .route("/", get(assets::landing))
        .route("/squire_web_bg.wasm", get(assets::get_wasm))
        .route("/squire_web.js", get(assets::get_js))
        .fallback(assets::landing)
        .with_state(state)
}

#[shuttle_runtime::main]
async fn axum(#[shuttle_shared_db::MongoDb] db_conn: Database) -> shuttle_axum::ShuttleAxum {
    let app_state = AppStateBuilder::with_db(db_conn).build();
    Ok(create_router(app_state).into())
}

#![cfg(feature = "db-tests")]
// TODO force tests to run independently (crate `serial_test`?)

use async_session::chrono::Utc;
use squire_sdk::{
    server::state::ServerState,
    tournaments::{TournOp, TournamentManager},
};

use crate::state::AppState;

async fn clear_database() {
    AppState::new().await.get_db().drop(None).await;
}

#[tokio::test]
async fn insert_fetch_tourn() {
    clear_database().await;

    let manager = TournamentManager::new(squire_tests::spoof_account(), squire_tests::get_seed());
    let state = AppState::new().await;

    state.persist_tourn(&manager).await;
    let retrieved_tourn = state
        .get_tourn(manager.id)
        .await
        .expect("Could not retrieve tournament from database");

    assert_eq!(manager, retrieved_tourn);
}

#[tokio::test]
async fn check_already_persisted() {
    clear_database().await;

    let mut manager =
        TournamentManager::new(squire_tests::spoof_account(), squire_tests::get_seed());
    let state = AppState::new().await;

    assert!(!state.persist_tourn(&manager).await);
    assert!(state.persist_tourn(&manager).await);
}

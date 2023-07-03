#![cfg(feature = "db-tests")]

use async_session::chrono::Utc;
use squire_sdk::{
    server::state::ServerState,
    tournaments::{TournOp, TournamentManager},
};

use crate::state::{AppSettings, AppState};

async fn clear_database() {
    AppState::new().await.get_db().drop(None).await;
}

#[tokio::test]
async fn insert_fetch_tourn() {
    clear_database().await;

    let manager = TournamentManager::new(squire_tests::spoof_account(), squire_tests::get_seed());
    let state = AppState::new_with_settings(
        AppSettings::default().database_name("SquireTesting_insert_fetch_tourn"),
    )
    .await;

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
    let state = AppState::new_with_settings(
        AppSettings::default().database_name("SquireTesting_check_already_persisted"),
    )
    .await;

    assert!(!state.persist_tourn(&manager).await);
    assert!(state.persist_tourn(&manager).await);
}

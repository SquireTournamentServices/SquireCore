use axum::{extract::State, response::IntoResponse, Json};
use squire_sdk::api::*;

use crate::state::AppState;

pub async fn create_account(
    State(_state): State<AppState>,
    Json(_account): Json<CreateAccount>,
) -> impl IntoResponse {
    todo!()
}

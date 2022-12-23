use axum::http::Request;
use hyper::Body;

use squire_sdk::{
    accounts::{CreateAccountRequest, CreateAccountResponse, LoginRequest, SquireAccountId},
    model::tournament::TournamentPreset,
    tournaments::CreateTournamentRequest,
};

use crate::tests::utils::create_request;

pub(crate) fn register_account_request() -> Request<Body> {
    let body = CreateAccountRequest {
        user_name: "Test User".into(),
        display_name: "Test".into(),
    };
    create_request("register", body)
}

pub(crate) fn login_request(id: SquireAccountId) -> Request<Body> {
    let body = LoginRequest { id };
    create_request("login", body)
}

pub(crate) fn create_tournament_request() -> Request<Body> {
    let body = CreateTournamentRequest {
        name: "Test".into(),
        preset: TournamentPreset::Swiss,
        format: "Pioneer".into(),
    };
    create_request("tournaments/create", body)
}

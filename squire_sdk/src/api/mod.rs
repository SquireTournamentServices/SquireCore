use serde::{Deserialize, Serialize};

use crate::{extend, sync::TournamentManager};

mod model;
mod request;
mod url;
pub use model::*;
pub use request::*;
pub use url::Url;

/* ---------- Base Routes ---------- */
const API_BASE: Url<0> = Url::from("/api/v1");

/* ---------- Tournament Routes ---------- */
const TOURNAMENTS_ROUTE: Url<0> = extend!(API_BASE, "/tournaments");

const GET_TOURNAMENT_ENDPOINT: Url<1> = Url::new("/:t_id", [":t_id"]);

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTournament;

impl GetRequest<1> for GetTournament {
    type Response = TournamentManager;
    const ROUTE: Url<1> = extend!(TOURNAMENTS_ROUTE, GET_TOURNAMENT_ENDPOINT);
}

const LIST_TOURNAMENTS_ENDPOINT: Url<1> = Url::new("/list/:page", [":page"]);

#[derive(Debug, Serialize, Deserialize)]
pub struct ListTournaments;

impl GetRequest<1> for ListTournaments {
    type Response = Vec<TournamentSummary>;
    const ROUTE: Url<1> = extend!(TOURNAMENTS_ROUTE, LIST_TOURNAMENTS_ENDPOINT);
}

const SUBSCRIBE_ENDPOINT: Url<1> = Url::new("/subscribe/:t_id", [":t_id"]);

#[derive(Debug, Serialize, Deserialize)]
pub struct Subscribe;

impl GetRequest<1> for Subscribe {
    type Response = ();
    const ROUTE: Url<1> = extend!(TOURNAMENTS_ROUTE, SUBSCRIBE_ENDPOINT);
}

const IMPORT_TOURN_ENDPOINT: Url<0> = Url::from("/");

impl PostRequest<0> for TournamentManager {
    type Response = bool;
    const ROUTE: Url<0> = extend!(TOURNAMENTS_ROUTE, IMPORT_TOURN_ENDPOINT);
}

/* ---------- Account Routes ---------- */
pub const ACCOUNTS_ROUTE: Url<0> = extend!(API_BASE, "/accounts");

/* ---------- Misc Routes ---------- */
pub const VERSION_ENDPOINT: Url<0> = Url::from("/version");

#[derive(Debug, Serialize, Deserialize)]
pub struct GetVersion;

impl GetRequest<0> for GetVersion {
    type Response = Version;
    const ROUTE: Url<0> = extend!(API_BASE, VERSION_ENDPOINT);
}

#[cfg(test)]
mod tests {
    use crate::api::*;

    #[test]
    fn verify_tournament_endpoints() {
        assert_eq!(GET_TOURNAMENT_ENDPOINT.as_str(), "/:t_id");
    }

    #[test]
    fn verify_tournament_routes() {
        assert_eq!(
            <GetTournament as GetRequest<1>>::ROUTE.as_str(),
            "/api/v1/tournaments/:t_id"
        );
        assert_eq!(
            <ListTournaments as GetRequest<1>>::ROUTE.as_str(),
            "/api/v1/tournaments/list/:page"
        );
    }

    #[test]
    fn verify_misc_endpoints() {}

    #[test]
    fn verify_misc_routes() {
        assert_eq!(
            <GetVersion as GetRequest<0>>::ROUTE.as_str(),
            "/api/v1/version"
        );
    }
}

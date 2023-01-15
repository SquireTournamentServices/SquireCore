use const_format::concatcp;

use crate::utils::Url;

pub const TOURNAMENTS_ROUTE: &str = "/api/v1/tournaments";

pub(crate) const CREATE_TOURNAMENT_ENDPOINT: Url<0> = Url::from("/create");
pub const CREATE_TOURNAMENT_ROUTE: Url<0> = Url::from(concatcp!(
    TOURNAMENTS_ROUTE,
    CREATE_TOURNAMENT_ENDPOINT.route
));

pub(crate) const GET_TOURNAMENT_ENDPOINT: Url<1> = Url::new("/:t_id", [":t_id"]);
pub const GET_TOURNAMENT_ROUTE: Url<1> = Url::new(
    concatcp!(TOURNAMENTS_ROUTE, GET_TOURNAMENT_ENDPOINT.route),
    GET_TOURNAMENT_ENDPOINT.replacements,
);

pub(crate) const SYNC_TOURNAMENT_ENDPOINT: Url<1> = Url::new(
    concatcp!(GET_TOURNAMENT_ENDPOINT.route, "/sync"),
    GET_TOURNAMENT_ENDPOINT.replacements,
);
pub const SYNC_TOURNAMENT_ROUTE: Url<1> = Url::new(
    concatcp!(TOURNAMENTS_ROUTE, SYNC_TOURNAMENT_ENDPOINT.route),
    SYNC_TOURNAMENT_ENDPOINT.replacements,
);

pub(crate) const ROLLBACK_TOURNAMENT_ENDPOINT: Url<1> = Url::new(
    concatcp!(GET_TOURNAMENT_ENDPOINT.route, "/rollback"),
    SYNC_TOURNAMENT_ENDPOINT.replacements,
);
pub const ROLLBACK_TOURNAMENT_ROUTE: Url<1> = Url::new(
    concatcp!(TOURNAMENTS_ROUTE, ROLLBACK_TOURNAMENT_ENDPOINT.route),
    ROLLBACK_TOURNAMENT_ENDPOINT.replacements,
);

pub static ACCOUNTS_ROUTE: &str = "/api/v1/accounts";

#[cfg(test)]
mod tests {
    use crate::api::{
        CREATE_TOURNAMENT_ENDPOINT, CREATE_TOURNAMENT_ROUTE, GET_TOURNAMENT_ENDPOINT,
        GET_TOURNAMENT_ROUTE, ROLLBACK_TOURNAMENT_ENDPOINT, ROLLBACK_TOURNAMENT_ROUTE,
        SYNC_TOURNAMENT_ENDPOINT, SYNC_TOURNAMENT_ROUTE,
    };

    #[test]
    fn verify_tournament_endpoints() {
        assert_eq!(CREATE_TOURNAMENT_ENDPOINT.as_str(), "/create");
        assert_eq!(GET_TOURNAMENT_ENDPOINT.as_str(), "/:t_id");
        assert_eq!(SYNC_TOURNAMENT_ENDPOINT.as_str(), "/:t_id/sync");
        assert_eq!(ROLLBACK_TOURNAMENT_ENDPOINT.as_str(), "/:t_id/rollback");
    }

    #[test]
    fn verify_tournament_routes() {
        assert_eq!(
            CREATE_TOURNAMENT_ROUTE.as_str(),
            "/api/v1/tournaments/create"
        );
        assert_eq!(GET_TOURNAMENT_ROUTE.as_str(), "/api/v1/tournaments/:t_id");
        assert_eq!(
            SYNC_TOURNAMENT_ROUTE.as_str(),
            "/api/v1/tournaments/:t_id/sync"
        );
        assert_eq!(
            ROLLBACK_TOURNAMENT_ROUTE.as_str(),
            "/api/v1/tournaments/:t_id/rollback"
        );
    }
}

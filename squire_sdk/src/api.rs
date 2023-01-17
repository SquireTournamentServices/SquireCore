use const_format::concatcp;

use crate::utils::Url;

/* ---------- Tournament Routes ---------- */
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

/* ---------- Account Routes ---------- */
pub const ACCOUNTS_ROUTE: &str = "/api/v1/accounts";

pub(crate) const REGISTER_ACCOUNT_ENDPOINT: Url<0> = Url::from("/register");
pub const REGISTER_ACCOUNT_ROUTE: Url<0> =
    Url::from(concatcp!(ACCOUNTS_ROUTE, REGISTER_ACCOUNT_ENDPOINT.route));

pub(crate) const VERIFY_ACCOUNT_ENDPOINT: Url<0> = Url::from("/verify");
pub const VERIFY_ACCOUNT_ROUTE: Url<0> =
    Url::from(concatcp!(ACCOUNTS_ROUTE, VERIFY_ACCOUNT_ENDPOINT.route));

pub(crate) const LOGIN_ENDPOINT: Url<0> = Url::from("/login");
pub const LOGIN_ROUTE: Url<0> =
    Url::from(concatcp!(ACCOUNTS_ROUTE, LOGIN_ENDPOINT.route));

pub(crate) const LOGOUT_ENDPOINT: Url<0> = Url::from("/logout");
pub const LOGOUT_ROUTE: Url<0> = Url::from(concatcp!(ACCOUNTS_ROUTE, LOGOUT_ENDPOINT.route));

pub(crate) const LOAD_ACCOUNT_ENDPOINT: Url<0> = Url::from("/load");
pub const LOAD_ACCOUNT_ROUTE: Url<0> =
    Url::from(concatcp!(ACCOUNTS_ROUTE, LOAD_ACCOUNT_ENDPOINT.route));

/* ---------- Misc Routes ---------- */
pub const VERSION_ROUTE: &str = "/api/v1/version";

#[cfg(test)]
mod tests {
    use crate::api::{
        CREATE_TOURNAMENT_ENDPOINT, CREATE_TOURNAMENT_ROUTE, GET_TOURNAMENT_ENDPOINT,
        GET_TOURNAMENT_ROUTE, LOAD_ACCOUNT_ENDPOINT, LOAD_ACCOUNT_ROUTE, LOGOUT_ENDPOINT,
        LOGOUT_ROUTE, REGISTER_ACCOUNT_ENDPOINT, REGISTER_ACCOUNT_ROUTE,
        ROLLBACK_TOURNAMENT_ENDPOINT, ROLLBACK_TOURNAMENT_ROUTE, SYNC_TOURNAMENT_ENDPOINT,
        SYNC_TOURNAMENT_ROUTE, VERIFY_ACCOUNT_ENDPOINT, VERIFY_ACCOUNT_ROUTE,
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

    #[test]
    fn verify_account_endpoints() {
        assert_eq!(REGISTER_ACCOUNT_ENDPOINT.as_str(), "/register");
        assert_eq!(VERIFY_ACCOUNT_ENDPOINT.as_str(), "/verify");
        assert_eq!(LOGOUT_ENDPOINT.as_str(), "/logout");
        assert_eq!(LOAD_ACCOUNT_ENDPOINT.as_str(), "/load");
    }

    #[test]
    fn verify_account_routes() {
        assert_eq!(REGISTER_ACCOUNT_ROUTE.as_str(), "/api/v1/accounts/register");
        assert_eq!(VERIFY_ACCOUNT_ROUTE.as_str(), "/api/v1/accounts/verify");
        assert_eq!(LOGOUT_ROUTE.as_str(), "/api/v1/accounts/logout");
        assert_eq!(LOAD_ACCOUNT_ROUTE.as_str(), "/api/v1/accounts/load");
    }
}

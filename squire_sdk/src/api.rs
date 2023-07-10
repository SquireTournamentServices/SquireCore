use crate::{extend, utils::Url};

/* ---------- Base Routes ---------- */
pub const API_BASE: Url<0> = Url::from("/api/v1");

/* ---------- Tournament Routes ---------- */
pub const TOURNAMENTS_ROUTE: Url<0> = extend!(API_BASE, "/tournaments");

pub(crate) const GET_TOURNAMENT_ENDPOINT: Url<1> = Url::new("/:t_id", [":t_id"]);
pub const GET_TOURNAMENT_ROUTE: Url<1> = extend!(TOURNAMENTS_ROUTE, GET_TOURNAMENT_ENDPOINT);

pub(crate) const LIST_TOURNAMENTS_ENDPOINT: Url<1> = Url::new("/list/:page", [":page"]);
pub const LIST_TOURNAMENTS_ROUTE: Url<1> = extend!(TOURNAMENTS_ROUTE, LIST_TOURNAMENTS_ENDPOINT);

pub(crate) const SUBSCRIBE_ENDPOINT: Url<1> = Url::new("/subscribe/:t_id", [":t_id"]);
pub const SUBSCRIBE_ROUTE: Url<1> = extend!(TOURNAMENTS_ROUTE, SUBSCRIBE_ENDPOINT);

/* ---------- Account Routes ---------- */
pub const ACCOUNTS_ROUTE: Url<0> = extend!(API_BASE, "/accounts");

pub(crate) const REGISTER_ACCOUNT_ENDPOINT: Url<0> = Url::from("/register");
pub const REGISTER_ACCOUNT_ROUTE: Url<0> = extend!(ACCOUNTS_ROUTE, REGISTER_ACCOUNT_ENDPOINT);

pub(crate) const VERIFY_ACCOUNT_ENDPOINT: Url<0> = Url::from("/verify");
pub const VERIFY_ACCOUNT_ROUTE: Url<0> = extend!(ACCOUNTS_ROUTE, VERIFY_ACCOUNT_ENDPOINT);

pub(crate) const LOGIN_ENDPOINT: Url<0> = Url::from("/login");
pub const LOGIN_ROUTE: Url<0> = extend!(ACCOUNTS_ROUTE, LOGIN_ENDPOINT);

pub(crate) const LOGOUT_ENDPOINT: Url<0> = Url::from("/logout");
pub const LOGOUT_ROUTE: Url<0> = extend!(ACCOUNTS_ROUTE, LOGOUT_ENDPOINT);

pub(crate) const LOAD_ACCOUNT_ENDPOINT: Url<0> = Url::from("/load");
pub const LOAD_ACCOUNT_ROUTE: Url<0> = extend!(ACCOUNTS_ROUTE, LOAD_ACCOUNT_ENDPOINT);

/* ---------- Misc Routes ---------- */
pub const VERSION_ENDPOINT: Url<0> = Url::from("/version");
pub const VERSION_ROUTE: Url<0> = extend!(API_BASE, VERSION_ENDPOINT);

#[cfg(test)]
mod tests {
    use crate::api::{
        GET_TOURNAMENT_ENDPOINT, GET_TOURNAMENT_ROUTE, LIST_TOURNAMENTS_ROUTE,
        LOAD_ACCOUNT_ENDPOINT, LOAD_ACCOUNT_ROUTE, LOGOUT_ENDPOINT, LOGOUT_ROUTE,
        REGISTER_ACCOUNT_ENDPOINT, REGISTER_ACCOUNT_ROUTE, VERIFY_ACCOUNT_ENDPOINT,
        VERIFY_ACCOUNT_ROUTE, VERSION_ROUTE,
    };

    #[test]
    fn verify_tournament_endpoints() {
        assert_eq!(GET_TOURNAMENT_ENDPOINT.as_str(), "/:t_id");
    }

    #[test]
    fn verify_tournament_routes() {
        assert_eq!(GET_TOURNAMENT_ROUTE.as_str(), "/api/v1/tournaments/:t_id");
        assert_eq!(
            LIST_TOURNAMENTS_ROUTE.as_str(),
            "/api/v1/tournaments/list/:page"
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

    #[test]
    fn verify_misc_endpoints() {}

    #[test]
    fn verify_misc_routes() {
        assert_eq!(VERSION_ROUTE.as_str(), "/api/v1/version");
    }
}

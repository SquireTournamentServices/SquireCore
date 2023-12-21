use derive_more::From;
use http::StatusCode;
use squire_lib::error::TournamentError;

use crate::compat::NetworkError;

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Debug, From)]
pub enum ClientError {
    NotLoggedIn,
    LogInFailed,
    FailedToConnect,
    Network(NetworkError),
    RequestStatus(StatusCode),
    Tournament(TournamentError),
}

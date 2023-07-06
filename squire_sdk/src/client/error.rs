use reqwest::StatusCode;
use squire_lib::error::TournamentError;

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Debug)]
pub enum ClientError {
    NotLoggedIn,
    LogInFailed,
    FailedToConnect,
    Reqwest(reqwest::Error),
    RequestStatus(StatusCode),
    Tournament(TournamentError),
}

impl From<StatusCode> for ClientError {
    fn from(status: StatusCode) -> Self {
        Self::RequestStatus(status)
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(error: reqwest::Error) -> Self {
        Self::Reqwest(error)
    }
}

impl From<TournamentError> for ClientError {
    fn from(error: TournamentError) -> Self {
        Self::Tournament(error)
    }
}

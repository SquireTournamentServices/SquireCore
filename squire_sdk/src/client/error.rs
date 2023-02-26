use reqwest::StatusCode;

#[derive(Debug)]
pub enum ClientError {
    Reqwest(reqwest::Error),
    RequestStatus(StatusCode),
    NotLoggedIn,
    LogInFailed,
    FailedToConnect,
}

use std::fmt::Debug;

use derive_more::From;
use futures::{Future, SinkExt};
use reqwest::{
    header::{HeaderMap, HeaderName},
    Client, Error as ReqwestError, Request, Response,
};
use squire_lib::{accounts::SquireAccount, tournament::TournamentId};

use super::{
    session::{SessionBroadcaster, SessionWatcher},
    HOST_ADDRESS,
};
use crate::{
    actor::*,
    api::{
        Credentials, GetRequest, GuestSession, Login, PostRequest, SessionToken, TokenParseError,
    },
    compat::{NetworkResponse, Websocket, WebsocketMessage, log},
};

#[cfg(target_family = "wasm")]
fn do_wrap<T>(value: T) -> send_wrapper::SendWrapper<T> {
    send_wrapper::SendWrapper::new(value)
}

#[cfg(not(target_family = "wasm"))]
const fn do_wrap<T>(value: T) -> T {
    value
}

pub type ReqwestResult<T> = Result<T, reqwest::Error>;
pub type NetworkClient = ActorClient<NetworkState>;

#[derive(Debug)]
pub struct NetworkState {
    url: String,
    session: SessionBroadcaster,
    token: Option<SessionToken>,
    client: Client,
}

/// Encapsulates all of the ways that a login attempt can fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginError {
    /// There was a network failure and the request (likely) did not reach the server.
    NetworkError,
    /// The server responded back saying that the given username and/or password are incorrect.
    CredentialError,
    /// There was an issue with the server... Unlikely to happen and should probably be logged
    /// somehow.
    ServerError,
}

#[derive(From)]
pub enum NetworkCommand {
    Request(Request, OneshotSender<NetworkResponse>),
    Login(Credentials, OneshotSender<Result<SquireAccount, LoginError>>),
    LoginComplete(Option<(SquireAccount, SessionToken)>),
    GuestLogin(OneshotSender<SessionWatcher>),
    GuestLoginComplete(Option<SessionToken>, OneshotSender<SessionWatcher>),
    OpenWebsocket(TournamentId, OneshotSender<Option<Websocket>>),
}

#[async_trait]
impl ActorState for NetworkState {
    type Message = NetworkCommand;

    async fn start_up(&mut self, _scheduler: &mut Scheduler<Self>) {
        // TODO: The browser should store a cookie. We should ping the server to get the session
        // info. If that fails, we should ping the server for an guest session.
        let token = self
            .post_request(GuestSession, [])
            .await
            .ok()
            .and_then(|resp| SessionToken::try_from(resp.headers()).ok());
        self.token = token;
        self.session.guest_auth();
    }

    async fn process(&mut self, scheduler: &mut Scheduler<Self>, msg: Self::Message) {
        match msg {
            NetworkCommand::Request(mut req, send) => {
                let headers = req.headers_mut();
                if let Some((name, header)) = self.token.as_ref().map(SessionToken::as_header) {
                    let _ = headers.insert(name, header);
                }
                let fut = self.client.execute(req);
                scheduler.process(async move { drop(send.send(NetworkResponse::new(fut.await))) });
            }
            NetworkCommand::Login(cred, send) => {
                let req = self.post_request(Login(cred), []);
                scheduler.add_task(async move {
                    let Ok(resp) = req.await else {
                        // FIXME: Don't assume it was a cred error. Look at the error and
                        // investigate.
                        drop(send.send(Err(LoginError::CredentialError)));
                        log("Request failed...");
                        return None;
                    };
                    let Some(token) = SessionToken::try_from(resp.headers()).ok() else {
                        drop(send.send(Err(LoginError::ServerError)));
                        log("Could not construct session token...");
                        return None;
                    };
                    let Some(acc) = resp.json::<SquireAccount>().await.ok() else {
                        drop(send.send(Err(LoginError::ServerError)));
                        log("Could not deserialize account...");
                        return None;
                    };
                    drop(send.send(Ok(acc.clone())));
                    Some((acc, token))
                });
            }
            NetworkCommand::LoginComplete(digest) => {
                if let Some((acc, token)) = digest {
                    self.token = Some(token);
                    self.session.user_auth(acc);
                }
            }
            NetworkCommand::GuestLogin(send) => {
                let req = self.post_request(GuestSession, []);
                scheduler.add_task(async move {
                    let digest = match req.await {
                        Ok(resp) => SessionToken::try_from(resp.headers()).ok(),
                        Err(_) => None,
                    };
                    (digest, send)
                });
            }
            NetworkCommand::GuestLoginComplete(token, send) => {
                self.token = token;
                self.session.guest_auth();
                drop(send.send(self.session.subscribe()))
            }
            NetworkCommand::OpenWebsocket(id, send) => match self.token.clone() {
                Some(token) => {
                    let url = format!("ws{HOST_ADDRESS}/api/v1/tournaments/subscribe/{id}");
                    scheduler.process(async move {
                        drop(send.send(init_ws(Websocket::new(&url).await.ok(), token).await));
                    });
                }
                None => drop(send.send(None)),
            },
        }
    }
}

impl NetworkState {
    pub fn new(url: String) -> Self {
        Self {
            url,
            session: SessionBroadcaster::new(),
            client: Client::new(),
            token: None,
        }
    }

    pub fn new_with_user(url: String, user: SquireAccount) -> Self {
        Self {
            url,
            session: SessionBroadcaster::new_with_user(user),
            client: Client::new(),
            token: None,
        }
    }

    pub fn subscribe(&self) -> SessionWatcher {
        self.session.subscribe()
    }

    pub fn get_request<const N: usize, R>(
        &self,
        subs: [&str; N],
    ) -> impl 'static + Send + Future<Output = Result<R::Response, ReqwestError>>
    where
        R: GetRequest<N>,
        R::Response: 'static + Send,
    {
        let url = format!("{}{}", self.url, R::ROUTE.replace(subs));
        let req = self.client.get(url);
        do_wrap(async move { req.send().await?.json().await })
    }

    pub fn post_request<const N: usize, B>(
        &self,
        body: B,
        subs: [&str; N],
    ) -> impl 'static + Send + Future<Output = Result<Response, ReqwestError>>
    where
        B: 'static + Send + Sync + PostRequest<N>,
    {
        let mut builder = self
            .client
            .post(format!("{}{}", self.url, B::ROUTE.replace(subs)))
            .header(HeaderName::from_static("content-type"), "application/json");
        if let Some((name, header)) = self.token.as_ref().map(SessionToken::as_header) {
            builder = builder.header(name.as_str(), header.to_str().unwrap());
        }
        let body = serde_json::to_string(&body).unwrap();
        do_wrap(async move { builder.body(body).send().await })
    }

    pub fn json_post_request<const N: usize, B>(
        &self,
        body: B,
        subs: [&str; N],
    ) -> impl 'static + Send + Future<Output = Result<B::Response, ReqwestError>>
    where
        B: 'static + Send + Sync + PostRequest<N>,
        B::Response: 'static + Send,
    {
        let resp = self.post_request(body, subs);
        do_wrap(async move { resp.await?.json().await })
    }
}

async fn init_ws(mut ws: Option<Websocket>, token: SessionToken) -> Option<Websocket> {
    if let Some(ws) = ws.as_mut() {
        let msg = WebsocketMessage::Bytes(postcard::to_allocvec(&token).unwrap());
        ws.send(msg).await.ok()?;
    }
    ws
}

impl Debug for NetworkCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkCommand::Request(_, _) => write!(f, "NetworkCommand::Request(..)"),
            NetworkCommand::Login(cred, _) => write!(f, "NetworkCommand::Login({cred:?})"),
            NetworkCommand::GuestLogin(_) => write!(f, "NetworkCommand::GuestLogin"),
            NetworkCommand::LoginComplete(login_comp) => {
                write!(f, "NetworkCommand::LoginComplete({login_comp:?})")
            }
            NetworkCommand::GuestLoginComplete(guest_login_comp, _) => write!(
                f,
                "NetworkCommand::GuestLoginComplete({guest_login_comp:?})"
            ),
            NetworkCommand::OpenWebsocket(id, _) => {
                write!(f, "NetworkCommand::OpenWebsocket({id})")
            }
        }
    }
}

impl From<((), OneshotSender<SessionWatcher>)> for NetworkCommand {
    fn from(((), send): ((), OneshotSender<SessionWatcher>)) -> Self {
        NetworkCommand::GuestLogin(send)
    }
}

impl TryFrom<&HeaderMap> for SessionToken {
    type Error = TokenParseError;

    fn try_from(headers: &HeaderMap) -> Result<Self, Self::Error> {
        match headers
            .get(Self::HEADER_NAME.as_str())
            .and_then(|h| h.to_str().ok())
        {
            Some(header) => header.parse(),
            None => Err(TokenParseError::NoAuthHeader),
        }
    }
}

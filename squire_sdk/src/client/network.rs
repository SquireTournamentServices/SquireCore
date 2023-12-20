use std::fmt::Debug;

use derive_more::From;
use futures::SinkExt;
use squire_lib::{accounts::SquireAccount, tournament::TournamentId};
use troupe::{prelude::*, compat::{SendableFuture, Sendable}};

use super::session::{SessionBroadcaster, SessionWatcher};
use crate::{
    api::{Credentials, GuestSession, Login, PostRequest, SessionToken},
    compat::{NetworkResponse, Websocket, WebsocketMessage, log, Client, Request, Response, NetworkError},
};

pub type NetworkClient = SinkClient<Permanent, NetworkCommand>;

#[derive(Debug)]
pub struct NetworkState {
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
    Login(
        Credentials,
        OneshotSender<Result<SquireAccount, LoginError>>,
    ),
    LoginComplete(Option<(SquireAccount, SessionToken)>),
    GuestLogin(OneshotSender<SessionWatcher>),
    GuestLoginComplete(Option<SessionToken>, OneshotSender<SessionWatcher>),
    OpenWebsocket(TournamentId, OneshotSender<Option<Websocket>>),
}

#[async_trait]
impl ActorState for NetworkState {
    type Permanence = Permanent;
    type ActorType = SinkActor;

    type Message = NetworkCommand;
    type Output = ();

    async fn start_up(&mut self, _scheduler: &mut Scheduler<Self>) {
        // TODO: The browser should store a cookie. We should ping the server to get the session
        // info. If that fails, we should ping the server for an guest session.
        let token = self
            .post_request(GuestSession, [])
            .await
            .ok()
            .and_then(|resp| resp.get_header(SessionToken::HEADER_NAME.as_str()))
            .and_then(|h| h.parse::<SessionToken>().ok());
        self.token = token;
        self.session.guest_auth();
    }

    async fn process(&mut self, scheduler: &mut Scheduler<Self>, msg: Self::Message) {
        match msg {
            NetworkCommand::Request(req, send) => {
                let fut = self.client.execute(req.session(self.token.as_ref()));
                scheduler.manage_future(async move { drop(send.send(NetworkResponse::new(fut.await))) });
            }
            NetworkCommand::Login(cred, send) => {
                let req = self.post_request(Login(cred), []);
                scheduler.queue_task(async move {
                    let Ok(resp) = req.await else {
                        // FIXME: Don't assume it was a cred error. Look at the error and
                        // investigate.
                        drop(send.send(Err(LoginError::CredentialError)));
                        log("Request failed...");
                        return None;
                    };
                    let Ok(token) = resp.session_token() else {
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
                scheduler.queue_task(async move {
                    let digest = match req.await {
                        Ok(resp) => resp.session_token().ok(),
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
                    let url = format!("/api/v1/tournaments/subscribe/{id}");
                    scheduler.manage_future(async move {
                        drop(send.send(init_ws(Websocket::new(&url).await.ok(), token).await));
                    });
                }
                None => drop(send.send(None)),
            },
        }
    }
}

impl NetworkState {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            session: SessionBroadcaster::new(),
            client: Client::new(),
            token: None,
        }
    }

    pub fn new_with_user(user: SquireAccount) -> Self {
        Self {
            session: SessionBroadcaster::new_with_user(user),
            client: Client::new(),
            token: None,
        }
    }

    pub fn subscribe(&self) -> SessionWatcher {
        self.session.subscribe()
    }

    pub fn post_request<const N: usize, B>(
        &self,
        body: B,
        subs: [&str; N],
    ) -> impl SendableFuture<Output = Result<Response, NetworkError>>
    where
        B: Sendable + PostRequest<N>,
    {
        let req = Request::post(&B::ROUTE.replace(subs))
            .session(self.token.as_ref())
            .json(&body);
        self.client.execute(req)
    }

    pub fn json_post_request<const N: usize, B>(
        &self,
        body: B,
        subs: [&str; N],
    ) -> impl SendableFuture<Output = Result<B::Response, NetworkError>>
    where
        B: 'static + Send + Sync + PostRequest<N>,
        B::Response: 'static + Send,
    {
        let resp = self.post_request(body, subs);
        async move { resp.await?.json().await }
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

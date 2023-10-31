use std::fmt::Debug;

use futures::{Future, SinkExt};
use http::header::CONTENT_TYPE;
use reqwest::{Client, Error as ReqwestError, Request, Response};
use squire_lib::{accounts::SquireAccount, tournament::TournamentId};

use super::{
    session::{SessionBroadcaster, SessionWatcher},
    HOST_ADDRESS,
};
use crate::{
    actor::*,
    api::{Credentials, GetRequest, GuestSession, Login, PostRequest, SessionToken},
    compat::{NetworkResponse, SendableWrapper, Websocket, WebsocketMessage},
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

pub enum NetworkCommand {
    Request(Request, OneshotSender<NetworkResponse>),
    Login(Credentials, OneshotSender<SessionWatcher>),
    LoginComplete(
        Option<(SquireAccount, SessionToken)>,
        OneshotSender<SessionWatcher>,
    ),
    GuestLogin(OneshotSender<SessionWatcher>),
    GuestLoginComplete(Option<SessionToken>, OneshotSender<SessionWatcher>),
    OpenWebsocket(TournamentId, OneshotSender<Option<Websocket>>),
}

#[async_trait]
impl ActorState for NetworkState {
    type Message = SendableWrapper<NetworkCommand>;

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
        match msg.take() {
            NetworkCommand::Request(req, send) => {
                let fut = self.client.execute(req);
                scheduler.process(async move { drop(send.send(NetworkResponse::new(fut.await))) });
            }
            NetworkCommand::Login(cred, send) => {
                let req = self.post_request(Login(cred), []);
                scheduler.add_task(async move {
                    let digest = match req.await {
                        Ok(resp) => match SessionToken::try_from(resp.headers()).ok() {
                            Some(token) => resp.json().await.ok().map(|acc| (acc, token)),
                            None => None,
                        },
                        Err(_) => None,
                    };
                    SendableWrapper::new(NetworkCommand::LoginComplete(digest, send))
                });
            }
            NetworkCommand::LoginComplete(digest, send) => {
                if let Some((acc, token)) = digest {
                    self.token = Some(token);
                    self.session.user_auth(acc);
                }
                drop(send.send(self.session.subscribe()))
            }
            NetworkCommand::GuestLogin(send) => {
                let req = self.post_request(GuestSession, []);
                scheduler.add_task(async move {
                    let digest = match req.await {
                        Ok(resp) => SessionToken::try_from(resp.headers()).ok(),
                        Err(_) => None,
                    };
                    SendableWrapper::new(NetworkCommand::GuestLoginComplete(digest, send))
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
            .header(CONTENT_TYPE, "application/json");
        if let Some((name, header)) = self.token.as_ref().map(SessionToken::as_header) {
            builder = builder.header(name, header);
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
            NetworkCommand::LoginComplete(login_comp, _) => {
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

impl Trackable<Credentials, SessionWatcher> for SendableWrapper<NetworkCommand> {
    fn track(cred: Credentials, send: OneshotSender<SessionWatcher>) -> Self {
        SendableWrapper::new(NetworkCommand::Login(cred, send))
    }
}

impl Trackable<(), SessionWatcher> for SendableWrapper<NetworkCommand> {
    fn track((): (), send: OneshotSender<SessionWatcher>) -> Self {
        SendableWrapper::new(NetworkCommand::GuestLogin(send))
    }
}

impl Trackable<TournamentId, Option<Websocket>> for SendableWrapper<NetworkCommand> {
    fn track(id: TournamentId, send: OneshotSender<Option<Websocket>>) -> Self {
        SendableWrapper::new(NetworkCommand::OpenWebsocket(id, send))
    }
}

impl Trackable<Request, NetworkResponse> for SendableWrapper<NetworkCommand> {
    fn track(req: Request, send: OneshotSender<NetworkResponse>) -> Self {
        SendableWrapper::new(NetworkCommand::Request(req, send))
    }
}

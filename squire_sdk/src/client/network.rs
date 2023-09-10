use async_trait::async_trait;
use futures::{Future, FutureExt};
use http::header::CONTENT_TYPE;
use reqwest::{Client, Error as ReqwestError, Response};
use squire_lib::accounts::SquireAccount;

use super::session::{SessionBroadcaster, SessionWatcher};
use crate::{
    actor::{ActorClient, ActorState, Scheduler},
    api::{Credentials, GetRequest, GuestSession, Login, PostRequest, SessionToken},
};

#[cfg(target_family = "wasm")]
fn do_wrap<T>(value: T) -> send_wrapper::SendWrapper<T> {
    send_wrapper::SendWrapper::new(value)
}

#[cfg(not(target_family = "wasm"))]
fn do_wrap<T>(value: T) -> T {
    value
}

pub type NetworkClient = ActorClient<NetworkState>;

pub struct NetworkState {
    url: String,
    session: SessionBroadcaster,
    token: Option<SessionToken>,
    client: Client,
}

pub enum NetworkCommand {
    Query(Box<dyn 'static + Send + FnOnce(&NetworkState)>),
    Login(Credentials),
    GuestLogin,
    LoginComplete(Result<(SquireAccount, Option<SessionToken>), ReqwestError>),
    GuestLoginComplete(Result<Option<SessionToken>, ReqwestError>),
}

#[async_trait]
impl ActorState for NetworkState {
    type Message = NetworkCommand;

    async fn process(&mut self, scheduler: &mut Scheduler<Self>, msg: Self::Message) {
        match msg {
            NetworkCommand::Query(query) => query(self),
            NetworkCommand::Login(cred) => {
                scheduler.add_task(self.post_request(Login(cred), []).then(process_login));
            }
            NetworkCommand::GuestLogin => {
                scheduler.add_task(self.post_request(GuestSession, []).map(process_guest_login));
            }
            NetworkCommand::LoginComplete(res) => {
                if let Ok((acc, token)) = res {
                    self.token = token;
                    self.session.user_auth(acc);
                }
            }
            NetworkCommand::GuestLoginComplete(res) => {
                if let Ok(token) = res {
                    self.token = token;
                    self.session.guest_auth();
                }
            }
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

fn process_login(
    res: Result<Response, ReqwestError>,
) -> impl 'static + Send + Future<Output = NetworkCommand> {
    let digest = async move {
        match res {
            Ok(resp) => {
                let token = SessionToken::try_from(resp.headers()).ok();
                NetworkCommand::LoginComplete(resp.json().await.map(move |acc| (acc, token)))
            }
            Err(err) => NetworkCommand::LoginComplete(Err(err)),
        }
    };
    do_wrap(digest)
}

fn process_guest_login(res: Result<Response, ReqwestError>) -> NetworkCommand {
    match res {
        Ok(resp) => {
            NetworkCommand::GuestLoginComplete(Ok(SessionToken::try_from(resp.headers()).ok()))
        }
        Err(err) => NetworkCommand::GuestLoginComplete(Err(err)),
    }
}

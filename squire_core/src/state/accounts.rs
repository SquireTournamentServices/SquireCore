use std::collections::HashMap;

use axum::response::{IntoResponse, Response};
use cycle_map::CycleMap;
use http::StatusCode;
use squire_sdk::{
    actor::*,
    api::{Credentials, RegForm},
    model::{accounts::SquireAccount, identifiers::SquireAccountId},
};

pub struct LoginError;

impl IntoResponse for LoginError {
    fn into_response(self) -> Response {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

#[derive(Debug, Clone)]
pub struct AccountStoreHandle {
    client: ActorClient<AccountStore>,
}

impl AccountStoreHandle {
    pub fn new() -> Self {
        let client = ActorClient::builder(AccountStore::new()).launch();
        Self { client }
    }

    pub fn create(&self, item: RegForm) -> Tracker<SquireAccountId> {
        self.client.track(item)
    }

    pub fn authenticate(&self, item: Credentials) -> Tracker<Option<SquireAccountId>> {
        self.client.track(item)
    }

    pub fn get(&self, item: SquireAccountId) -> Tracker<Option<SquireAccount>> {
        self.client.track(item)
    }

    pub fn delete(&self, item: SquireAccountId) -> Tracker<bool> {
        self.client.track(item)
    }
}

pub enum AccountCommand {
    Create(RegForm, OneshotSender<SquireAccountId>),
    Authenticate(Credentials, OneshotSender<Option<SquireAccountId>>),
    Get(SquireAccountId, OneshotSender<Option<SquireAccount>>),
    Delete(SquireAccountId, OneshotSender<bool>),
}

#[derive(Debug, Default)]
pub struct AccountStore {
    credentials: CycleMap<Credentials, SquireAccountId>,
    users: HashMap<SquireAccountId, SquireAccount>,
}

#[async_trait]
impl ActorState for AccountStore {
    type Message = AccountCommand;

    async fn process(&mut self, _scheduler: &mut Scheduler<Self>, msg: Self::Message) {
        match msg {
            AccountCommand::Create(form, send) => {
                let _ = send.send(self.create_account(form));
            }
            AccountCommand::Authenticate(cred, send) => {
                let _ = send.send(self.authenticate(cred));
            }
            AccountCommand::Get(id, send) => {
                let _ = send.send(self.get_account(id));
            }
            AccountCommand::Delete(id, send) => {
                let _ = send.send(self.delete_account(id));
            }
        }
    }
}

impl AccountStore {
    fn new() -> Self {
        Self {
            users: HashMap::new(),
            credentials: CycleMap::new(),
        }
    }

    fn create_account(&mut self, form: RegForm) -> SquireAccountId {
        let cred: Credentials = form.clone().into();
        if let Some(id) = self.credentials.get_right(&cred) {
            return *id;
        }
        let RegForm {
            username,
            display_name,
            ..
        } = form;
        let acc = SquireAccount::new(username, display_name);
        let digest = acc.id;
        self.credentials.insert(cred, digest);
        self.users.insert(digest, acc);
        digest
    }

    fn authenticate(&mut self, cred: Credentials) -> Option<SquireAccountId> {
        self.credentials.get_right(&cred).cloned()
    }

    fn get_account(&mut self, id: SquireAccountId) -> Option<SquireAccount> {
        self.users.get(&id).cloned()
    }

    fn delete_account(&mut self, id: SquireAccountId) -> bool {
        let digest = self.users.remove(&id).is_some();
        self.credentials.remove_via_right(&id);
        digest
    }
}

impl Trackable<SquireAccountId, Option<SquireAccount>> for AccountCommand {
    fn track(id: SquireAccountId, send: OneshotSender<Option<SquireAccount>>) -> Self {
        Self::Get(id, send)
    }
}

impl Trackable<SquireAccountId, bool> for AccountCommand {
    fn track(msg: SquireAccountId, send: OneshotSender<bool>) -> Self {
        Self::Delete(msg, send)
    }
}

impl Trackable<Credentials, Option<SquireAccountId>> for AccountCommand {
    fn track(msg: Credentials, send: OneshotSender<Option<SquireAccountId>>) -> Self {
        Self::Authenticate(msg, send)
    }
}

impl Trackable<RegForm, SquireAccountId> for AccountCommand {
    fn track(msg: RegForm, send: OneshotSender<SquireAccountId>) -> Self {
        Self::Create(msg, send)
    }
}


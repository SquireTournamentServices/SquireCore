use std::collections::HashMap;

use axum::response::{IntoResponse, Response};
use cycle_map::CycleMap;
use http::StatusCode;
use squire_sdk::{
    api::{Credentials, RegForm},
    model::{accounts::SquireAccount, identifiers::SquireAccountId},
};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    oneshot::{channel as oneshot_channel, Sender as OneshotSender},
};

use super::Tracker;

pub struct LoginError;

impl IntoResponse for LoginError {
    fn into_response(self) -> Response {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

#[derive(Debug, Clone)]
pub struct AccountStoreHandle {
    handle: UnboundedSender<AccountCommand>,
}

impl AccountStoreHandle {
    pub fn new() -> Self {
        let (send, recv) = unbounded_channel();
        let store = AccountStore::new(recv);
        tokio::spawn(store.run());
        Self { handle: send }
    }

    pub fn create(&self, item: RegForm) -> Tracker<SquireAccountId> {
        let (send, recv) = oneshot_channel();
        let _ = self.handle.send(AccountCommand::Create(item, send));
        Tracker::new(recv)
    }

    pub fn authenticate(&self, item: Credentials) -> Tracker<Option<SquireAccountId>> {
        let (send, recv) = oneshot_channel();
        let _ = self.handle.send(AccountCommand::Authenticate(item, send));
        Tracker::new(recv)
    }

    pub fn get(&self, item: SquireAccountId) -> Tracker<Option<SquireAccount>> {
        let (send, recv) = oneshot_channel();
        let _ = self.handle.send(AccountCommand::Get(item, send));
        Tracker::new(recv)
    }

    pub fn delete(&self, item: SquireAccountId) -> Tracker<bool> {
        let (send, recv) = oneshot_channel();
        let _ = self.handle.send(AccountCommand::Delete(item, send));
        Tracker::new(recv)
    }
}

pub enum AccountCommand {
    Create(RegForm, OneshotSender<SquireAccountId>),
    Authenticate(Credentials, OneshotSender<Option<SquireAccountId>>),
    Get(SquireAccountId, OneshotSender<Option<SquireAccount>>),
    Delete(SquireAccountId, OneshotSender<bool>),
}

#[derive(Debug)]
pub struct AccountStore {
    inbound: UnboundedReceiver<AccountCommand>,
    credentials: CycleMap<Credentials, SquireAccountId>,
    users: HashMap<SquireAccountId, SquireAccount>,
}

impl AccountStore {
    fn new(inbound: UnboundedReceiver<AccountCommand>) -> Self {
        Self {
            inbound,
            users: HashMap::new(),
            credentials: CycleMap::new(),
        }
    }

    async fn run(mut self) -> ! {
        loop {
            match self.inbound.recv().await.unwrap() {
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

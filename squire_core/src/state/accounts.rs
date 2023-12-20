use std::{collections::HashMap, future::Future, hash::Hasher};

use async_trait::async_trait;
use axum::response::{IntoResponse, Response};
use derive_more::From;
use futures::{FutureExt, StreamExt};
use fxhash::FxHasher;
use http::StatusCode;
use mongodb::{
    bson::{doc, Document},
    options::{UpdateModifications, UpdateOptions},
    Collection, Database,
};
use serde::{Deserialize, Serialize};
use squire_sdk::{
    api::{Credentials, RegForm},
    model::{accounts::SquireAccount, identifiers::SquireAccountId},
};
use tokio::sync::oneshot::Sender as OneshotSender;
use tracing::Level;
use troupe::{prelude::*, sink::permanent::Tracker};

pub struct LoginError;

impl IntoResponse for LoginError {
    fn into_response(self) -> Response {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

#[derive(Debug, Clone)]
pub struct AccountStoreHandle {
    client: SinkClient<Permanent, AccountCommand>,
}

fn salt_and_hash(password: &str, username: &str) -> u32 {
    let mut hasher = FxHasher::default();
    hasher.write(password.as_bytes());
    hasher.write(username.as_bytes());
    let hash = hasher.finish().to_be_bytes();
    u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]])
}

impl AccountStoreHandle {
    pub fn new(db: Database) -> Self {
        let client = ActorBuilder::new(AccountStore::new(db)).launch();
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

#[derive(From)]
pub enum AccountCommand {
    Create(RegForm, OneshotSender<SquireAccountId>),
    Authenticate(Credentials, OneshotSender<Option<SquireAccountId>>),
    Get(SquireAccountId, OneshotSender<Option<SquireAccount>>),
    Delete(SquireAccountId, OneshotSender<bool>),
}

#[derive(Debug)]
pub struct AccountStore {
    credentials: HashMap<u32, SquireAccountId>,
    users: HashMap<SquireAccountId, DbUser>,
    db: AccountDb,
}

#[async_trait]
impl ActorState for AccountStore {
    type Permanence = Permanent;
    type ActorType = SinkActor;

    type Message = AccountCommand;
    type Output = ();

    async fn start_up(&mut self, _scheduler: &mut Scheduler<Self>) {
        let db = self.db.clone();
        db.load_all_accounts(self).await;
    }

    async fn process(&mut self, scheduler: &mut Scheduler<Self>, msg: Self::Message) {
        match msg {
            AccountCommand::Authenticate(cred, send) => drop(send.send(self.authenticate(cred))),
            AccountCommand::Get(id, send) => drop(send.send(self.get_account(id))),
            AccountCommand::Delete(id, send) => drop(send.send(self.delete_account(id, scheduler))),
            AccountCommand::Create(form, send) => {
                let _ = send.send(self.create_account(form, scheduler));
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AccountDb {
    db: Database,
}

impl AccountStore {
    fn new(db: Database) -> Self {
        Self {
            users: HashMap::new(),
            credentials: HashMap::new(),
            db: AccountDb::new(db),
        }
    }

    fn create_account(
        &mut self,
        form: RegForm,
        scheduler: &mut Scheduler<Self>,
    ) -> SquireAccountId {
        let cred: Credentials = form.clone().into();
        let Credentials::Basic { username, password } = cred;
        let cred = salt_and_hash(&password, &username);
        if let Some(id) = self.credentials.get(&cred) {
            return *id;
        }
        let RegForm {
            username,
            display_name,
            ..
        } = form;
        let account = SquireAccount::new(username, display_name);
        let digest = account.id;
        let user = DbUser { account, cred };
        scheduler.manage_future(self.db.persist_account(user.clone()));
        self.credentials.insert(cred, digest);
        self.users.insert(digest, user);
        digest
    }

    fn authenticate(&mut self, cred: Credentials) -> Option<SquireAccountId> {
        let Credentials::Basic { username, password } = cred;
        let hash = salt_and_hash(&password, &username);
        self.credentials.get(&hash).cloned()
    }

    fn get_account(&mut self, id: SquireAccountId) -> Option<SquireAccount> {
        self.users.get(&id).map(|user| &user.account).cloned()
    }

    fn delete_account(&mut self, id: SquireAccountId, scheduler: &mut Scheduler<Self>) -> bool {
        self.credentials.retain(|_, a_id| id != *a_id);
        if let Some(user) = self.users.remove(&id) {
            scheduler.manage_future(self.db.remove_account(user));
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbUser {
    account: SquireAccount,
    /// The salted and hashed password.
    cred: u32,
}

impl AccountDb {
    const ACCOUNTS_TABLE: &'static str = "Accounts";

    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn get_table(&self) -> Collection<DbUser> {
        self.db.collection(Self::ACCOUNTS_TABLE)
    }

    #[allow(dead_code)]
    pub async fn load_all_accounts(&self, cache: &mut AccountStore) {
        let mut cursor = self.get_table().find(None, None).await.unwrap();
        while let Some(acc) = cursor.next().await {
            if let Ok(user) = acc {
                cache.credentials.insert(user.cred, user.account.id);
                cache.users.insert(user.account.id, user);
            }
        }
    }

    fn persist_account(&self, acc: DbUser) -> impl 'static + Future<Output = ()> {
        let table = self.get_table();
        persist_account(table, acc).map(drop)
    }

    fn remove_account(&self, acc: DbUser) -> impl 'static + Future<Output = ()> {
        let table = self.get_table();
        delete_account(table, acc).map(drop)
    }
}

async fn persist_account(table: Collection<DbUser>, account: DbUser) -> bool {
    println!("Saving user: {account:?}");
    let doc: Document = mongodb::bson::to_raw_document_buf(&account)
        .unwrap()
        .try_into()
        .unwrap();
    if table
        .update_one(
            doc.clone(),
            UpdateModifications::Document(doc! {"$set": doc}),
            UpdateOptions::builder().upsert(true).build(),
        )
        .await
        .is_err()
    {
        if let Err(err) = table.insert_one(account.clone(), None).await {
            tracing::event!(
                Level::WARN,
                "Could not persist session `{account:?}` got error: {err}",
            );
            return false;
        }
    }
    true
}

async fn delete_account(table: Collection<DbUser>, account: DbUser) -> bool {
    let doc: Document = mongodb::bson::to_raw_document_buf(&account)
        .unwrap()
        .try_into()
        .unwrap();
    table.delete_one(doc, None).await.is_ok()
}

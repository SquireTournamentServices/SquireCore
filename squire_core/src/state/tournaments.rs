use std::{sync::Arc, ops::Range};

use futures::StreamExt;
use mongodb::{
    bson::{doc, spec::BinarySubtype, Binary, Document},
    Collection, Database, options::{UpdateModifications, UpdateOptions, Hint, FindOptions},
};
use squire_sdk::{
    actor::*, model::tournament::TournamentId, server::gathering::PersistMessage,
    sync::TournamentManager, api::TournamentSummary,
};
use tracing::Level;

#[derive(Debug, Clone)]
pub struct TournDb {
    db_conn: Database,
    tourn_coll: Arc<str>,
}

pub struct TournPersister {
    db: TournDb,
}

#[async_trait]
impl ActorState for TournPersister {
    type Message = PersistMessage;

    async fn process(&mut self, _scheduler: &mut Scheduler<Self>, msg: Self::Message) {
        match msg {
            PersistMessage::Get(id, send) => {
                let _ = send.send(self.get_tourn(id).await);
            }
            PersistMessage::Persist(tourn) => {
                self.db.persist_tourn(&tourn).await;
            }
        }
    }
}

impl TournPersister {
    pub fn new(db: TournDb) -> Self {
        Self { db }
    }

    pub async fn get_tourn(&self, id: TournamentId) -> Option<Box<TournamentManager>> {
        self.db.get_tourn(id).await
    }
}

impl TournDb {
    const TOURN_INDEX_NAME: &str = "tourn_id";

    pub fn new(db_conn: Database, tourn_coll: Arc<str>) -> Self {
        Self {
            db_conn,
            tourn_coll,
        }
    }

    pub fn get_db(&self) -> Database {
        self.db_conn.clone()
    }

    pub fn get_tourns(&self) -> Collection<TournamentManager> {
        self.get_db().collection(&self.tourn_coll)
    }

    fn make_query(id: TournamentId) -> Document {
        doc! { "tourn.id": Binary {
            bytes: id.as_bytes().to_vec(),
            subtype: BinarySubtype::Generic,
        }}
    }

    pub async fn get_tourn(&self, id: TournamentId) -> Option<Box<TournamentManager>> {
        self.get_tourns()
            .find_one(Some(Self::make_query(id)), None)
            .await
            .ok()
            .flatten()
            .map(Box::new)
    }

    pub async fn persist_tourn(&self, tourn: &TournamentManager) -> bool {
        // There appears to be a problem in bson right now where `Collection::replace_one` uses the
        // normal document serializer, but `Collection::find_one` (and `Collection::insert_one` as
        // well) use the raw document serializer, which unfortunately behave differently. Therefore
        // `Collection::update_one` is used as a workaround so that we can call the raw document
        // serializer here
        let doc: Document = mongodb::bson::to_raw_document_buf(tourn)
            .unwrap()
            .try_into()
            .unwrap();
        match self
            .get_tourns()
            .update_one(
                Self::make_query(tourn.id),
                UpdateModifications::Document(doc! {"$set": doc}),
                UpdateOptions::builder()
                    .upsert(true)
                    .hint(Hint::Name(Self::TOURN_INDEX_NAME.to_string()))
                    .build(),
            )
            .await
        {
            Ok(result) => result.matched_count != 0,
            Err(_) => match self.get_tourns().insert_one(tourn, None).await {
                Ok(_) => true,
                Err(err) => {
                    tracing::event!(
                        Level::WARN,
                        r#"Could not persist tournament with name "{}" and id "{}" due to error: {err}"#,
                        tourn.tourn().name,
                        tourn.tourn().id,
                    );
                    false
                }
            },
        }
    }

    pub async fn get_tourn_summaries(&self, including: Range<usize>) -> Vec<TournamentSummary> {
        let Ok(cursor) = self
            .get_tourns()
            .find(
                None,
                FindOptions::builder().sort(doc! {"$natural":-1}).build(),
            )
            .await
        else {
            return vec![];
        };

        cursor
            .skip(including.start)
            .take(including.count())
            .filter_map(|u| async { u.ok().as_ref().map(TournamentSummary::from) })
            .collect()
            .await
    }
}

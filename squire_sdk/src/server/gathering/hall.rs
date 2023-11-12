use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use axum::extract::ws::WebSocket;
use instant::{Duration, Instant};
use squire_lib::tournament::TournamentId;
use tokio::sync::oneshot::Sender as OneshotSender;

use super::{Gathering, GatheringMessage, PersistMessage};
use crate::{
    actor::{ActorBuilder, ActorClient, ActorState, Scheduler},
    server::session::SessionWatcher,
    sync::TournamentManager,
};

/* TODO:
 *  - Clients might close their websocket communicates. Because of this, we need to ensure that the
 *  `SelectAll` stream doesn't yield empty frames endlessly.
 *    - Similarly, we need to understand the behavior if all stream close
 *
 *  - The GatheringHall needs access to the server state (or at least a way to fetch/write
 *  tournament data)
 *
 *  - The GatheringHall needs a way to query the Gatherings for their current tournament state for
 *  persistence (likely another channel).
 *
 *  - Gatherings need a way to disperse. This likely happens when a tournament ends.
 *
 *  - WebSocket messages are paired with user data. Currently, that data is just a user account. Updates to the tournament are associated with
 *  that user data. We need to ensure the following are upheld while validating tournament updates:
 *    - The data does not need to be updated over the lifetime of the Gathering (accounts should be
 *    good here)
 *    - The user data is retrieved from a user's session so that we know the caller is who they say
 *    they are (again, accounts should be good here)
 *
 */

fn schedule_persist(scheduler: &mut Scheduler<GatheringHall>) {
    scheduler.schedule(
        Instant::now() + Duration::from_secs(5),
        GatheringHallMessage::Persist,
    );
}

/// A message sent to a `GatheringHall` that communicates some command that it needs to process.
#[derive(Debug)]
pub enum GatheringHallMessage {
    /// Create a new gathering
    NewGathering(TournamentId),
    /// Adds an onlooker to a gathering
    NewConnection(TournamentId, SessionWatcher, WebSocket),
    ///
    PersistReady(TournamentId),
    /// Persist all the tournaments that need to be persisted
    Persist,
    /// Destroy a gathering when the gathering decides that it should be terminated
    DestroyGathering(TournamentId, OneshotSender<()>),
}

/// This structure manages all of the `Gathering`s around tournaments. This includes adding new
/// users to different gatherings and persisting data to the database. All of this is handled
/// through message passing and tokio tasks.
#[derive(Debug)]
pub struct GatheringHall {
    gatherings: HashMap<TournamentId, ActorClient<GatheringMessage>>,
    persister: ActorClient<PersistMessage>,
    to_persist: HashSet<TournamentId>,
}

#[async_trait]
impl ActorState for GatheringHall {
    type Message = GatheringHallMessage;

    async fn start_up(&mut self, scheduler: &mut Scheduler<Self>) {
        schedule_persist(scheduler);
    }

    async fn process(&mut self, scheduler: &mut Scheduler<Self>, msg: Self::Message) {
        match msg {
            GatheringHallMessage::NewGathering(id) => {
                let client = scheduler.client();
                let _ = self.process_new_gathering(id, client).await;
            }
            GatheringHallMessage::NewConnection(id, user, ws) => {
                let client = scheduler.client();
                let _ = self.process_new_onlooker(id, user, ws, client).await;
            }
            GatheringHallMessage::Persist => {
                let mut persist_reqs = HashMap::new();
                for id in self.to_persist.drain() {
                    if let Some(client) = self.gatherings.get_mut(&id) {
                        let tourn = client.track(()).await;
                        let _ = persist_reqs.insert(id, tourn);
                    }
                }

                persist_reqs
                    .drain()
                    .for_each(|(_, tourn)| self.persister.send(tourn));
                schedule_persist(scheduler);
            }
            GatheringHallMessage::DestroyGathering(id, sender) => {
                let _ = self.gatherings.remove(&id);
                let _ = sender.send(());
                // TODO(dora): We might need to do more bookkeeping on the destruction.
            }
            GatheringHallMessage::PersistReady(id) => {
                let _ = self.to_persist.insert(id);
            }
        }
    }
}

impl GatheringHall {
    /// Creates a new `GatheringHall` from receiver halves of channels that communicate new
    /// gatherings and subscriptions
    pub fn new(persister: ActorClient<PersistMessage>) -> Self {
        Self {
            gatherings: HashMap::new(),
            to_persist: HashSet::new(),
            persister,
        }
    }

    async fn spawn_gathering(
        &self,
        id: TournamentId,
        client: ActorClient<GatheringHallMessage>,
    ) -> Option<ActorClient<GatheringMessage>> {
        let tourn = self.get_tourn(&id).await?;

        let gathering = Gathering::new(*tourn, client);
        let client = ActorBuilder::new(gathering).launch();
        Some(client)
    }

    async fn process_new_gathering(
        &mut self,
        id: TournamentId,
        client: ActorClient<GatheringHallMessage>,
    ) {
        // TODO: We need a way to communicate that a tournament can not be found
        let Some(send) = self.spawn_gathering(id, client).await else {
            return;
        };
        _ = self.gatherings.insert(id, send);
    }

    async fn process_new_onlooker(
        &mut self,
        id: TournamentId,
        user: SessionWatcher,
        ws: WebSocket,
        client: ActorClient<GatheringHallMessage>,
    ) {
        let msg = GatheringMessage::NewConnection(user, ws);
        let send = self.get_or_init_gathering(id, client).await;
        send.send(msg)
    }

    async fn get_or_init_gathering(
        &mut self,
        id: TournamentId,
        client: ActorClient<GatheringHallMessage>,
    ) -> ActorClient<GatheringMessage> {
        if let Some(send) = self.gatherings.get(&id).cloned() {
            return send;
        }
        // FIXME: This can fail. We need a way to signal this possibility.
        let send = self.spawn_gathering(id, client).await.unwrap();
        _ = self.gatherings.insert(id, send.clone());
        send
    }

    async fn get_tourn(&self, id: &TournamentId) -> Option<Box<TournamentManager>> {
        match self.gatherings.get(id) {
            //  Ask the gathering for a copy of the tournament
            Some(handle) => Some(handle.track(()).await),
            None => self.persister.track(*id).await,
        }
    }
}

// impl Trackable<TournamentId, ()> for GatheringHallMessage {
//     fn track(msg: TournamentId, send: tokio::sync::oneshot::Sender<()>) -> Self {
//         Self::DestroyGathering(msg, send)
//     }
// }

impl From<(TournamentId, tokio::sync::oneshot::Sender<()>)> for GatheringHallMessage {
    fn from(msg: (TournamentId, tokio::sync::oneshot::Sender<()>)) -> Self {
        let (id, send) = msg;
        Self::DestroyGathering(id, send)
    }
}

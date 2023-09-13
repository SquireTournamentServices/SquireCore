use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use axum::extract::ws::WebSocket;
use instant::{Duration, Instant};
use squire_lib::tournament::TournamentId;
use tokio::sync::{
    mpsc::{channel, Receiver, Sender},
    oneshot::channel as oneshot_channel,
};

use super::{Gathering, GatheringMessage, PersistMessage};
use crate::{
    actor::{ActorBuilder, ActorClient, ActorState, Scheduler},
    api::AuthUser,
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

/// A message sent to a `GatheringHall` that communicates some command that it needs to process.
#[derive(Debug)]
pub enum GatheringHallMessage {
    /// Create a new gathering
    NewGathering(TournamentId),
    /// Adds an onlooker to a gathering
    NewConnection(TournamentId, AuthUser, WebSocket),
    /// Perist all the tournaments that need to be persisted
    Persist,
}

/// This structure manages all of the `Gathering`s around tournaments. This includes adding new
/// users to different gatherings and persisting data to the database. All of this is handled
/// through message passing and tokio tasks.
#[derive(Debug)]
pub struct GatheringHall {
    //state: S,
    gatherings: HashMap<TournamentId, ActorClient<Gathering>>,
    persists: Receiver<PersistMessage>,
    persist_sender: Sender<PersistMessage>,
}

#[async_trait]
impl ActorState for GatheringHall {
    type Message = GatheringHallMessage;

    async fn process(&mut self, scheduler: &mut Scheduler<Self>, msg: Self::Message) {
        match msg {
            GatheringHallMessage::NewGathering(id) => self.process_new_gathering(id).await,
            GatheringHallMessage::NewConnection(id, user, ws) => {
                self.process_new_onlooker(id, user, ws).await
            }
            GatheringHallMessage::Persist => {
                let mut to_persist = HashSet::new();
                let mut persist_reqs = HashMap::new();
                while let Ok(PersistMessage(id)) = self.persists.try_recv() {
                    let _ = to_persist.insert(id);
                }
                for id in to_persist.drain() {
                    let sender = self.gatherings.get_mut(&id).unwrap();
                    let (send, recv) = oneshot_channel();
                    let msg = GatheringMessage::GetTournament(send);
                    sender.send(msg);
                    let tourn = recv.await.unwrap();
                    let _ = persist_reqs.insert(id, tourn);
                }
                /*
                let _ = self
                    .state
                    .bulk_persist(persist_reqs.drain().map(|(_, tourn)| tourn))
                    .await;
                */
                scheduler.schedule(
                    Instant::now() + Duration::from_secs(5),
                    GatheringHallMessage::Persist,
                );
                todo!()
            }
        }
    }
}

impl GatheringHall {
    /// Creates a new `GatheringHall` from receiver halves of channels that communicate new
    /// gatherings and subscriptions
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let (persist_sender, persists) = channel(1000);
        Self {
            gatherings: HashMap::new(),
            persists,
            persist_sender,
        }
    }

    async fn spawn_gathering(&self, id: TournamentId) -> Option<ActorClient<Gathering>> {
        let tourn = self.get_tourn(&id).await?;
        let gathering = Gathering::new(tourn, self.persist_sender.clone());
        let client = ActorBuilder::new(gathering).launch();
        Some(client)
    }

    async fn process_new_gathering(&mut self, id: TournamentId) {
        // TODO: We need a way to communicate that a tournament can not be found
        let Some(send) = self.spawn_gathering(id).await else {
            return;
        };
        _ = self.gatherings.insert(id, send);
    }

    async fn process_new_onlooker(&mut self, id: TournamentId, user: AuthUser, ws: WebSocket) {
        let msg = GatheringMessage::NewConnection(user, ws);
        let send = self.get_or_init_gathering(id).await;
        send.send(msg)
    }

    async fn get_or_init_gathering(&mut self, id: TournamentId) -> ActorClient<Gathering> {
        if let Some(send) = self.gatherings.get(&id).cloned() {
            return send;
        }
        // FIXME: This can fail. We need a way to signal this possibility.
        let send = self.spawn_gathering(id).await.unwrap();
        _ = self.gatherings.insert(id, send.clone());
        send
    }

    async fn get_tourn(&self, id: &TournamentId) -> Option<TournamentManager> {
        match self.gatherings.get(id) {
            Some(handle) => {
                //  Ask the gathering for a copy of the tournament
                let (send, recv) = oneshot_channel();
                handle.send(GatheringMessage::GetTournament(send));
                recv.await.ok()
            }
            None => todo!(), //self.state.get_tourn(*id).await,
        }
    }
}

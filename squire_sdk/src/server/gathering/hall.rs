use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    hash::Hash,
    time::Duration,
};

use async_session::Session;
use axum::extract::ws::{Message, WebSocket};
use futures::{
    stream::{SelectAll, SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use squire_lib::tournament::TournamentId;
use tokio::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        oneshot::{channel as oneshot_channel, Sender as OneshotSender},
        OnceCell,
    },
    time::Instant,
};

use super::{Gathering, GatheringMessage, PersistMessage, ServerState, User};
use crate::sync::{
    ClientBound, ClientBoundMessage, OpSync, ServerBound, ServerBoundMessage, TournamentManager,
};

const GATHERING_HALL_CHANNEL_SIZE: usize = 100;

pub static GATHERING_HALL_MESSAGER: OnceCell<Sender<GatheringHallMessage>> = OnceCell::const_new();

/// This function spawns a tokio task to manage a gathering hall. It also sets up the necessary
/// channels for communicating with that task
pub fn init_gathering_hall<S: ServerState>(state: S) {
    let (send, recv) = channel(GATHERING_HALL_CHANNEL_SIZE);
    let hall = GatheringHall::new(state, recv);
    GATHERING_HALL_MESSAGER.set(send).unwrap();
    drop(tokio::spawn(hall.run()));
}

/// This function communicates with the gathering hall task to add a new onlooker into a
/// tournament.
pub async fn handle_new_onlooker(id: TournamentId, user: User, ws: WebSocket) {
    GATHERING_HALL_MESSAGER
        .get()
        .unwrap()
        .send(GatheringHallMessage::NewConnection(id, user, ws))
        .await
        .unwrap()
}

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
    NewConnection(TournamentId, User, WebSocket),
}

/// This structure manages all of the `Gathering`s around tournaments. This includes adding new
/// users to different gatherings and persisting data to the database. All of this is handled
/// through message passing and tokio tasks.
#[derive(Debug)]
pub struct GatheringHall<S> {
    state: S,
    gatherings: HashMap<TournamentId, Sender<GatheringMessage>>,
    inbound: Receiver<GatheringHallMessage>,
    persists: Receiver<PersistMessage>,
    persist_sender: Sender<PersistMessage>,
}

impl<S: ServerState> GatheringHall<S> {
    /// Creates a new `GatheringHall` from receiver halves of channels that communicate new
    /// gatherings and subscriptions
    pub fn new(state: S, inbound: Receiver<GatheringHallMessage>) -> Self {
        let (persist_sender, persists) = channel(1000);
        Self {
            gatherings: HashMap::new(),
            persists,
            persist_sender,
            inbound,
            state,
        }
    }

    pub async fn run(mut self) -> ! {
        let wait_time = Duration::from_secs(10);
        let now_then = || Instant::now().checked_add(wait_time).unwrap();
        let mut then = now_then();
        let mut to_persist = HashSet::new();
        let mut persist_reqs = HashMap::new();
        loop {
            tokio::select! {
                _ = tokio::time::sleep_until(then) => {
                    then = now_then();
                    while let Ok(PersistMessage(id)) = self.persists.try_recv() {
                        _ = to_persist.insert(id);
                    }
                    for id in to_persist.drain() {
                        let sender = self.gatherings.get_mut(&id).unwrap();
                        let (send, recv) = oneshot_channel();
                        let msg = GatheringMessage::GetTournament(send);
                        sender.send(msg).await;
                        let tourn = recv.await.unwrap();
                        _ = persist_reqs.insert(id, tourn);
                    }
                    _ = self.state.bulk_persist(persist_reqs.drain().map(|(_, tourn)| tourn)).await;
                }
                msg = self.inbound.recv() => match msg.unwrap() {
                    GatheringHallMessage::NewGathering(id) => self.process_new_gathering(id).await,
                    GatheringHallMessage::NewConnection(id, user, ws) => self.process_new_onlooker(id, user, ws).await,
                }
            }
        }
    }

    async fn spawn_gathering(&self, id: TournamentId) -> Option<Sender<GatheringMessage>> {
        let tourn = self.get_tourn(&id).await?;
        let (send, recv) = channel(100);
        let gathering = Gathering::new(tourn, recv, self.persist_sender.clone());
        drop(tokio::spawn(gathering.run()));
        Some(send)
    }

    async fn process_new_gathering(&mut self, id: TournamentId) {
        // TODO: We need a way to communicate that a tournament can not be found
        let Some(send) = self.spawn_gathering(id).await else {
            return;
        };
        _ = self.gatherings.insert(id, send);
    }

    async fn process_new_onlooker(&mut self, id: TournamentId, user: User, ws: WebSocket) {
        let msg = GatheringMessage::NewConnection(user, ws);
        if let Some(send) = self.get_or_init_gathering(id).await {
            send.send(msg).await.unwrap()
        }
    }

    async fn get_or_init_gathering(
        &mut self,
        id: TournamentId,
    ) -> Option<Sender<GatheringMessage>> {
        if let Some(send) = self.gatherings.get(&id).cloned() {
            return Some(send);
        }
        // FIXME: This can fail. We need a way to signal this possibility.
        let send = self.spawn_gathering(id).await?;
        _ = self.gatherings.insert(id, send.clone());
        Some(send)
    }

    async fn get_tourn(&self, id: &TournamentId) -> Option<TournamentManager> {
        match self.gatherings.get(id) {
            Some(handle) => {
                //  Ask the gathering for a copy of the tournament
                let (send, recv) = oneshot_channel();
                let _ = handle.send(GatheringMessage::GetTournament(send)).await;
                recv.await.ok()
            }
            None => self.state.get_tourn(*id).await,
        }
    }
}

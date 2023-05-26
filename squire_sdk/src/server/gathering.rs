use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    time::Duration,
};

use async_session::Session;
use axum::extract::ws::{Message, WebSocket};
use futures::{
    stream::{SelectAll, SplitSink, SplitStream},
    StreamExt,
};
use squire_lib::tournament::TournamentId;
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    time::Instant,
};

use crate::{tournaments::TournamentManager, sync::OpSync};

use super::User;

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

/// A message sent to a `GatheringHall` that communicates that a new `Gathering` needs to be
/// created.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewGatheringMessage(pub TournamentId);

/// A message sent to a `Gathering` that subscribes a new `Onlooker`.
#[derive(Debug)]
pub struct IncomingOnlookerMessage(pub TournamentId, NewOnlookerMessage);

/// A message sent to a `Gathering` that subscribes a new `Onlooker`.
#[derive(Debug)]
pub struct NewOnlookerMessage(pub User, pub WebSocket);

/// A message that communicates to the `GatheringHall` that it needs to backup tournament data.
/// How this data is backed up depends on the server implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PersistMessage(TournamentId);

/// This structure manages all of the `Gathering`s around tournaments. This includes adding new
/// users to different gatherings and persisting data to the database. All of this is handled
/// through message passing and tokio tasks.
pub struct GatheringHall {
    gatherings: HashMap<TournamentId, Sender<NewOnlookerMessage>>,
    onlookers: Receiver<IncomingOnlookerMessage>,
    new_gatherings: Receiver<NewGatheringMessage>,
    persists: Receiver<PersistMessage>,
    persist_sender: Sender<PersistMessage>,
}

/// This structure contains all users currently subscribed to a tournament and can be thought of as
/// the crowd of people that gathers for a tournament. New subscribers, called `Onlooker`s, are
/// added by the `GatheringHall` via a message passed through a channel. This structure is intented
/// to be consumed and turned into a tokio task. Inside that task, the Gathering listens to
/// messages coming from WebSockets, processes those messages, and the sends those messages back
/// out to all the other `Onlooker`s.
///
/// NOTE: The Gathering contains a copy of the tournament manager. This copy is the most up-to-date
/// version of the tournament. However, backup copies of the tournament are sent to `GatheringHall`
/// whenever there is a change and other locations upon request.
pub struct Gathering {
    tourn: TournamentManager,
    new_onlookers: Receiver<NewOnlookerMessage>,
    incoming: SelectAll<SplitStream<WebSocket>>,
    onlookers: HashMap<User, Onlooker>,
    persist: Sender<PersistMessage>,
}

/// This structure represents a user that is in some way participating in the tournament. This
/// person could be a spectator, player, judge, or admin. An onlooker represents two primary pieces
/// of information: a user (including session) and the websocket that they are listening on.
pub struct Onlooker(pub SplitSink<WebSocket, Message>);

impl GatheringHall {
    /// Creates a new `GatheringHall` from receiver halves of channels that communicate new
    /// gatherings and subscriptions
    pub fn new(
        onlookers: Receiver<IncomingOnlookerMessage>,
        new_gatherings: Receiver<NewGatheringMessage>,
    ) -> Self {
        let (persist_sender, persists) = channel(1000);
        Self {
            gatherings: HashMap::new(),
            new_gatherings,
            persists,
            persist_sender,
            onlookers,
        }
    }

    pub async fn run(mut self) -> ! {
        let wait_time = Duration::from_secs(5);
        let now_then = || Instant::now().checked_add(wait_time).unwrap();
        let mut then = now_then();
        let mut persist_reqs = HashSet::new();
        loop {
            tokio::select! {
                _ = tokio::time::sleep_until(then) => {
                    while let Ok(PersistMessage(id)) = self.persists.try_recv() {
                        persist_reqs.insert(id);
                    }
                    for id in persist_reqs.drain() {
                        todo!("Gathering hall collect tournament data for persistence");
                    }
                }
                msg = self.new_gatherings.recv() => {
                    self.process_new_gathering(msg.unwrap());
                }
                msg = self.onlookers.recv() => {
                    self.process_new_onlooker(msg.unwrap());
                }
            }
        }
    }

    async fn process_new_gathering(&mut self, msg: NewGatheringMessage) {
        let NewGatheringMessage(id) = msg;
        // TODO: We need a way to communicate that a tournament can not be found
        let Some(tourn) = self.get_tourn(&id).await else { return };
        let (send, recv) = channel(100);
        let gathering = Gathering::new(tourn, recv, self.persist_sender.clone());
        tokio::spawn(gathering.run());
        self.gatherings.insert(id, send);
    }

    async fn process_new_onlooker(&mut self, msg: IncomingOnlookerMessage) {
        let IncomingOnlookerMessage(id, msg) = msg;
        match self.gatherings.get(&id) {
            // TODO: Remove unwrap. Gathering might have imploded if the tournament has ended.
            Some(sender) => sender.send(msg).await.unwrap(),
            None => self.process_new_gathering(NewGatheringMessage(id)).await,
        }
    }

    async fn get_tourn(&self, id: &TournamentId) -> Option<TournamentManager> {
        todo!()
    }
}

impl Gathering {
    fn new(
        tourn: TournamentManager,
        new_onlookers: Receiver<NewOnlookerMessage>,
        persist: Sender<PersistMessage>,
    ) -> Self {
        let count = tourn.tourn().get_player_count();
        Self {
            tourn,
            new_onlookers,
            incoming: SelectAll::new(),
            onlookers: HashMap::with_capacity(count),
            persist,
        }
    }

    async fn run(mut self) -> ! {
        loop {
            if self.incoming.is_empty() {
                let msg = self.new_onlookers.recv().await;
                self.process_new_onlooker(msg.unwrap())
            } else {
                tokio::select! {
                    msg = self.new_onlookers.recv() => {
                        self.process_new_onlooker(msg.unwrap())
                    }
                    msg = self.incoming.next() => {
                        let res = match msg.unwrap() {
                            Ok(msg) => self.process_incoming_message(msg),
                            Err(_) => continue, // One of the streams closed... I think, move on
                        };
                        self.sync_request_response(res).await;
                    }
                }
            }
        }
    }

    fn process_new_onlooker(&mut self, msg: NewOnlookerMessage) {
        let NewOnlookerMessage(user, ws) = msg;
        let (sink, stream) = ws.split();
        let onlooker = Onlooker(sink);
        self.incoming.push(stream);
        match self.onlookers.get_mut(&user) {
            Some(ol) => *ol = onlooker,
            None => {
                self.onlookers.insert(user, onlooker);
            }
        }
    }

    // TODO: Return a "real" value
    fn process_incoming_message(&mut self, msg: Message) -> Result<(), ()> {
        let Message::Binary(data) = msg else { return Err(()) }; // We only process binary data
        let sync = postcard::from_bytes(&data).map_err(|_| ())?;
        self.validate_sync_request(&sync)?;
        self.process_sync_request(sync)
    }

    // TODO: Return an actual error
    // TODO: This method does not actually check to see if the person that sent the request is
    // allowed to send such a return. This will need to eventually change
    fn validate_sync_request(&mut self, sync: &OpSync) -> Result<(), ()> {
        Ok(())
    }

    // TODO: Return a "real" value
    fn process_sync_request(&mut self, sync: OpSync) -> Result<(), ()> {
        todo!()
    }

    /// This method handles dispatching the response to a sync request as well as any additional
    /// forwarding any necessary requests to the other `Onlooker`s
    async fn sync_request_response(&mut self, res: Result<(), ()>) {
        match res {
            Err(_) => {
                // Respond back with the sync error
            }
            Ok(_) => {
                // Respond back saying the sync was successful
                // Forward the sync request/list of ops to all other clients
            }
        }
        todo!()
    }

    fn disperse(self) {
        todo!()
    }
}

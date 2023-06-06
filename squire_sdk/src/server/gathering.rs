use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    hash::Hash,
    time::Duration,
};

use async_session::Session;
use axum::extract::ws::{Message, WebSocket};
use futures::{
    stream::{SelectAll, SplitSink, SplitStream},
    StreamExt, SinkExt,
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
use ulid::Ulid;

use crate::{sync::{OpSync, ServerBoundMessage, ClientBoundMessage, ClientBound, ServerBound}, tournaments::TournamentManager};

use super::{state::ServerState, User};

const GATHERING_HALL_CHANNEL_SIZE: usize = 100;

pub static GATHERING_HALL_MESSAGER: OnceCell<Sender<GatheringHallMessage>> = OnceCell::const_new();

/// This function spawns a tokio task to manage a gathering hall. It also sets up the necessary
/// channels for communicating with that task
pub fn init_gathering_hall<S: ServerState>(state: S) {
    let (send, recv) = channel(GATHERING_HALL_CHANNEL_SIZE);
    let hall = GatheringHall::new(state, recv);
    GATHERING_HALL_MESSAGER.set(send).unwrap();
    tokio::spawn(hall.run());
}

/// This function communicates with the gathering hall task to add a new onlooker into a
/// tournament.
pub async fn handle_new_onlooker(id: TournamentId, user: User, ws: WebSocket) {
    GATHERING_HALL_MESSAGER
        .get()
        .unwrap()
        .send(GatheringHallMessage::NewOnlooker(id, user, ws))
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
    NewOnlooker(TournamentId, User, WebSocket),
}

/// A message sent to a `Gathering` that subscribes a new `Onlooker`.
#[derive(Debug)]
pub enum GatheringMessage {
    GetTournament(OneshotSender<TournamentManager>),
    NewOnlooker(User, WebSocket),
}

/// A message that communicates to the `GatheringHall` that it needs to backup tournament data.
/// How this data is backed up depends on the server implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PersistMessage(TournamentId);

/// This structure manages all of the `Gathering`s around tournaments. This includes adding new
/// users to different gatherings and persisting data to the database. All of this is handled
/// through message passing and tokio tasks.
pub struct GatheringHall<S> {
    state: S,
    gatherings: HashMap<TournamentId, Sender<GatheringMessage>>,
    inbound: Receiver<GatheringHallMessage>,
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
    messages: Receiver<GatheringMessage>,
    ws_streams: SelectAll<SplitStream<WebSocket>>,
    onlookers: HashMap<User, Onlooker>,
    persist: Sender<PersistMessage>,
}

/// This structure represents a user that is in some way participating in the tournament. This
/// person could be a spectator, player, judge, or admin. An onlooker represents two primary pieces
/// of information: a user (including session) and the websocket that they are listening on.
pub struct Onlooker(pub SplitSink<WebSocket, Message>);

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
                msg = self.inbound.recv() => match msg.unwrap() {
                    GatheringHallMessage::NewGathering(id) => self.process_new_gathering(id).await,
                    GatheringHallMessage::NewOnlooker(id, user, ws) => self.process_new_onlooker(id, user, ws).await,
                }
            }
        }
    }

    async fn spawn_gathering(&self, id: TournamentId) -> Option<Sender<GatheringMessage>> {
        let tourn = self.get_tourn(&id).await?;
        let (send, recv) = channel(100);
        let gathering = Gathering::new(tourn, recv, self.persist_sender.clone());
        tokio::spawn(gathering.run());
        Some(send)
    }

    async fn process_new_gathering(&mut self, id: TournamentId) {
        // TODO: We need a way to communicate that a tournament can not be found
        let Some(send) = self.spawn_gathering(id).await else { return };
        self.gatherings.insert(id, send);
    }

    async fn process_new_onlooker(&mut self, id: TournamentId, user: User, ws: WebSocket) {
        let msg = GatheringMessage::NewOnlooker(user, ws);
        let send = self.get_or_init_gathering(id).await;
        send.send(msg).await.unwrap()
    }

    async fn get_or_init_gathering(&mut self, id: TournamentId) -> Sender<GatheringMessage> {
        if let Some(send) = self.gatherings.get(&id).cloned() {
            return send
        }
        // FIXME: This can fail. We need a way to signal this possibility.
        let send = self.spawn_gathering(id).await.unwrap();
        self.gatherings.insert(id, send.clone());
        send
    }

    async fn get_tourn(&self, id: &TournamentId) -> Option<TournamentManager> {
        match self.gatherings.get(&id) {
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

impl Gathering {
    fn new(
        tourn: TournamentManager,
        new_onlookers: Receiver<GatheringMessage>,
        persist: Sender<PersistMessage>,
    ) -> Self {
        let count = tourn.tourn().get_player_count();
        Self {
            tourn,
            messages: new_onlookers,
            ws_streams: SelectAll::new(),
            onlookers: HashMap::with_capacity(count),
            persist,
        }
    }

    async fn run(mut self) -> ! {
        println!("Running gathering for tournament: {}", self.tourn.id);
        loop {
            tokio::select! {
                msg = self.messages.recv() => {
                    println!("Internal message received.");
                    self.process_channel_message(msg.unwrap())
                }
                msg = self.ws_streams.next(), if !self.ws_streams.is_empty() => {
                    println!("Websocket message retreived.");
                    match msg.unwrap() {
                        Ok(msg) => {
                            self.process_incoming_message(msg).await;
                        },
                        Err(err) => {
                            println!("Message was an error... {err:?}");
                        }
                    };
                    println!("Websocket message processed!!");
                    //self.sync_request_response(res).await;
                }
            }
            println!("Processed message. Looping again...");
        }
    }

    fn process_channel_message(&mut self, msg: GatheringMessage) {
        println!("Processing internal message...");
        match msg {
            GatheringMessage::GetTournament(send) => send.send(self.tourn.clone()).unwrap(),
            GatheringMessage::NewOnlooker(user, ws) => {
                println!("Processing new onlooker...");
                let (sink, stream) = ws.split();
                let onlooker = Onlooker(sink);
                self.ws_streams.push(stream);
                match self.onlookers.get_mut(&user) {
                    Some(ol) => *ol = onlooker,
                    None => {
                        self.onlookers.insert(user, onlooker);
                    }
                }
                let resp = ServerBoundMessage {
                    id: Ulid::default(),
                    body: ServerBound::Fetch(self.tourn.id)
                };
                let resp_bytes = postcard::to_allocvec(&resp).unwrap();
                println!("New onlooker processed!! Send the following to see the tournament data:\n{resp_bytes:02X?}");
            }
        }
    }

    // TODO: Return a "real" value
    async fn process_incoming_message(&mut self, msg: Message) -> Result<(), ()> {
        println!("Processing incoming WS message...");
        let Message::Binary(data) = msg else { return Err(()) }; // We only process binary data
        let msg: ServerBoundMessage = postcard::from_bytes(&data).map_err(|_| ())?;
        println!("Deserialized message...");
        match msg.body {
            ServerBound::Fetch(id) => {
                println!("Fetch request found...");
                let resp = ClientBoundMessage {
                    id: msg.id,
                    body: ClientBound::FetchResp(self.tourn.clone())
                };
                let resp_bytes = postcard::to_allocvec(&resp).unwrap();
                println!("Sending tournament to everyone!!");
                for ol in self.onlookers.values_mut() {
                    ol.0.send(Message::Binary(resp_bytes.clone())).await.unwrap();
                }
            }
            ServerBound::SyncReq(_) => todo!(),
            ServerBound::SyncSeen => todo!(),
        }
        Ok(())
        /*
        self.validate_sync_request(&sync)?;
        self.process_sync_request(sync)
        */
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

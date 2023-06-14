use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    hash::Hash,
    sync::Arc,
    time::Duration,
};

use async_session::Session;
use axum::extract::ws::{Message, WebSocket};
use futures::{
    stream::{SelectAll, SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use squire_lib::{identifiers::SquireAccountId, tournament::TournamentId};
use tokio::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        oneshot::{channel as oneshot_channel, Sender as OneshotSender},
        OnceCell,
    },
    time::Instant,
};
use ulid::Ulid;

use crate::{
    sync::{ClientBound, ClientBoundMessage, OpSync, ServerBound, ServerBoundMessage},
    tournaments::TournamentManager,
};

use super::{state::ServerState, User};

mod hall;
mod onlooker;
pub use hall::*;
pub use onlooker::*;

/// A message sent to a `Gathering` that subscribes a new `Onlooker`.
#[derive(Debug)]
pub enum GatheringMessage {
    GetTournament(OneshotSender<TournamentManager>),
    NewConnection(User, WebSocket),
}

/// A message that communicates to the `GatheringHall` that it needs to backup tournament data.
/// How this data is backed up depends on the server implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PersistMessage(Arc<TournamentManager>);

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
    ws_streams: SelectAll<Crier>,
    onlookers: HashMap<SquireAccountId, Onlooker>,
    persist: Sender<PersistMessage>,
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
                msg = self.messages.recv() =>
                    self.process_channel_message(msg.unwrap()),
                msg = self.ws_streams.next(), if !self.ws_streams.is_empty() =>
                    self.process_websocket_message(msg).await,
            }
        }
    }

    async fn process_websocket_message(&mut self, msg: Option<(SquireAccountId, Option<Vec<u8>>)>) {
        match msg {
            Some((user, Some(bytes))) => {
                self.process_incoming_message(user, bytes).await;
            }
            Some((user, None)) => {
                self.onlookers.remove(&user);
            }
            None => {}
        }
    }

    fn process_channel_message(&mut self, msg: GatheringMessage) {
        match msg {
            GatheringMessage::GetTournament(send) => send.send(self.tourn.clone()).unwrap(),
            GatheringMessage::NewConnection(user, ws) => {
                let (sink, stream) = ws.split();
                let crier = Crier::new(stream, user.account.id);
                let onlooker = Onlooker::new(sink);
                self.ws_streams.push(crier);
                match self.onlookers.get_mut(&user.account.id) {
                    Some(ol) => *ol = onlooker,
                    None => {
                        self.onlookers.insert(user.account.id, onlooker);
                    }
                }
            }
        }
    }

    // TODO: Return a "real" value
    async fn process_incoming_message(&mut self, user: SquireAccountId, bytes: Vec<u8>) {
        let ServerBoundMessage { id, body } =
            match postcard::from_bytes::<ServerBoundMessage>(&bytes) {
                Ok(val) => val,
                Err(_) => todo!("Send a 'failed to deserialize message' to sender"),
            };
        match body {
            ServerBound::Fetch => {
                if let Some(user) = self.onlookers.get_mut(&user) {
                    let resp = ClientBoundMessage::new(ClientBound::FetchResp(self.tourn.clone()));
                    let _ = user.send_msg(&resp).await;
                }
            }
            _ => todo!(),
        }
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

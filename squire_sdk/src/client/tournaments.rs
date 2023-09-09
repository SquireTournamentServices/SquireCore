use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Debug,
};

use async_trait::async_trait;
use futures::{stream::SplitSink, SinkExt, StreamExt};
use instant::Instant;
use squire_lib::{
    operations::{OpData, OpResult, TournOp},
    tournament::TournamentId,
};
use tokio::sync::{
    oneshot::{self, channel as oneshot_channel, Sender as OneshotSender},
    watch::{channel as watch_channel, Receiver as Watcher, Sender as Broadcaster},
};
use uuid::Uuid;

use super::{OnUpdate, HOST_ADDRESS};
use crate::{
    actor::{ActorBuilder, ActorClient, ActorState, Scheduler, Trackable, Tracker},
    api::{GetRequest, Subscribe},
    compat::{Websocket, WebsocketError, WebsocketMessage, WebsocketResult},
    sync::{
        ClientBound, ClientBoundMessage, ClientForwardingManager, ClientOpLink, ClientSyncManager,
        OpSync, ServerBound, ServerBoundMessage, ServerOpLink, SyncForwardResp, TournamentManager,
        WebSocketMessage, RETRY_LIMIT,
    },
};

/// A container for the channels used to communicate with the tournament management task.
#[derive(Clone)]
pub struct TournsClient {
    client: ActorClient<ManagerState>,
}

pub(crate) enum ManagementCommand {
    Query(TournamentId, Query),
    Update(TournamentId, UpdateType, OneshotSender<Option<OpResult>>),
    Import(Box<TournamentManager>, OneshotSender<TournamentId>),
    Subscribe(TournamentId, OneshotSender<Option<Watcher<()>>>),
    Connection(
        Result<(Websocket, Box<TournamentManager>), ()>,
        OneshotSender<Option<Watcher<()>>>,
    ),
    Remote(WebsocketResult),
    Retry(MessageRetry),
}

impl Debug for ManagementCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManagementCommand::Query(id, _) => write!(f, "Query({id:?})"),
            ManagementCommand::Update(id, update, _) => write!(f, "Update({id:?}, {update:?})"),
            ManagementCommand::Import(tourn, _) => write!(f, "Import({tourn:?})"),
            ManagementCommand::Subscribe(id, _) => write!(f, "Subscribe({id:?})"),
            ManagementCommand::Connection(res, _) => write!(f, "Connection({:?})", res.is_ok()),
            ManagementCommand::Remote(res) => write!(f, "Remote({res:?}"),
            ManagementCommand::Retry(rt) => write!(f, "Retry({rt:?}"),
        }
    }
}

/// A struct that contains all of the state that the management task maintains
struct ManagerState {
    cache: TournamentCache,
    syncs: ClientSyncManager,
    forwarded: ClientForwardingManager,
    on_update: Box<dyn OnUpdate>,
}

#[async_trait]
impl ActorState for ManagerState {
    type Message = ManagementCommand;

    async fn process(&mut self, scheduler: &mut Scheduler<Self>, msg: Self::Message) {
        match msg {
            ManagementCommand::Query(id, query) => {
                self.handle_query(id, query);
            }
            ManagementCommand::Import(tourn, send) => {
                let _ = send.send(self.handle_import(*tourn));
            }
            ManagementCommand::Update(id, update, send) => {
                let _ = send.send(self.handle_update(scheduler, id, update).await);
            }
            ManagementCommand::Subscribe(id, send) => match self.handle_sub(id) {
                SubCreation::Connected(watcher) => {
                    let _ = send.send(Some(watcher));
                }
                SubCreation::Connect(id) => scheduler.add_task(create_ws_connection(id, send)),
            },
            ManagementCommand::Connection(res, send) => match res {
                Ok((ws, tourn)) => {
                    let _ = send.send(Some(self.handle_connection(scheduler, ws, tourn)));
                }
                Err(()) => {
                    let _ = send.send(None);
                }
            },
            ManagementCommand::Remote(ws_res) => match ws_res {
                Ok(msg) => drop(self.handle_ws_msg(scheduler, msg)),
                Err(err) => self.handle_ws_err(err),
            },
            ManagementCommand::Retry(MessageRetry { msg, id }) => {
                if self.syncs.is_latest_msg(&msg) {
                    if let Some(comm) = self.cache.get_mut(&id) {
                        comm.send(scheduler, msg).await
                    }
                }
            }
        }
    }
}

pub const MANAGEMENT_PANICKED_MSG: &str = "tournament management task panicked";

#[derive(Debug, Clone)]
pub enum UpdateType {
    Removal,
    Single(Box<TournOp>),
    Bulk(Vec<TournOp>),
}

type Query = Box<dyn Send + FnOnce(Option<&TournamentManager>)>;

impl TournsClient {
    pub fn new<O: OnUpdate>(on_update: O) -> Self {
        let client = ActorBuilder::new(ManagerState::new(on_update)).launch();
        Self { client }
    }

    pub fn import(&self, tourn: TournamentManager) -> Tracker<TournamentId> {
        self.client.track(tourn)
    }

    pub fn subscribe(&self, id: TournamentId) -> Tracker<Option<Watcher<()>>> {
        self.client.track(id)
    }

    pub fn query<F, T>(&self, id: TournamentId, query: F) -> Tracker<Option<T>>
    where
        F: 'static + Send + FnOnce(&TournamentManager) -> T,
        T: 'static + Send,
    {
        self.client.track((id, query))
    }

    pub async fn query_or_default<F, T>(&self, id: TournamentId, query: F) -> T
    where
        F: 'static + Send + FnOnce(&TournamentManager) -> T,
        T: 'static + Send + Default,
    {
        self.client.track((id, query)).await.unwrap_or_default()
    }

    pub fn update(&self, id: TournamentId, update: UpdateType) -> Tracker<Option<OpResult>> {
        self.client.track((id, update))
    }
}

/// Contains all the info needed to track a tournament and all outbound communication related to
/// it. Since not all tournaments have associated outbound communicate, the `comm` field is
/// optional.
#[derive(Debug)]
struct TournComm {
    tourn: TournamentManager,
    comm: Option<(SplitSink<Websocket, WebsocketMessage>, Broadcaster<()>)>,
}

type TournamentCache = HashMap<TournamentId, TournComm>;

enum SubCreation {
    Connected(Watcher<()>),
    Connect(TournamentId),
}

impl ManagerState {
    fn new<O: OnUpdate>(on_update: O) -> Self {
        Self {
            on_update: Box::new(on_update),
            cache: Default::default(),
            syncs: Default::default(),
            forwarded: Default::default(),
        }
    }

    fn handle_import(&mut self, tourn: TournamentManager) -> TournamentId {
        let id = tourn.id;
        let tc = TournComm { tourn, comm: None };
        _ = self.cache.insert(id, tc);
        id
    }

    async fn handle_update(
        &mut self,
        scheduler: &mut Scheduler<Self>,
        id: TournamentId,
        update: UpdateType,
    ) -> Option<OpResult> {
        let tourn = self.cache.get_mut(&id)?;
        let res = match update {
            UpdateType::Single(op) => tourn.tourn.apply_op(*op),
            UpdateType::Bulk(ops) => tourn.tourn.bulk_apply_ops(ops),
            UpdateType::Removal => {
                let _ = self.cache.remove(&id);
                return Some(Ok(OpData::Nothing));
            }
        };
        if res.is_ok() {
            (self.on_update)(id);
            let id = Uuid::new_v4();
            let sync: ClientOpLink = tourn.tourn.sync_request().into();
            self.syncs
                .initialize_chain(id, tourn.tourn.id, sync.clone())
                .unwrap(); // TODO: Remove unwrap
            let msg = ServerBoundMessage {
                id,
                body: sync.into(),
            };
            tourn.send(scheduler, msg).await;
        }
        Some(res)
    }

    fn handle_query(&self, id: TournamentId, query: Query) {
        query(self.cache.get(&id).map(|tc| &tc.tourn));
    }

    // Needs to take a &mut to the SelectAll WS listener so it can be updated if need be
    fn handle_sub(&mut self, id: TournamentId) -> SubCreation {
        match self.cache.get(&id) {
            Some(TournComm {
                comm: Some((_, broad)),
                ..
            }) => SubCreation::Connected(broad.subscribe()),
            _ => SubCreation::Connect(id),
        }
    }

    fn handle_connection(
        &mut self,
        scheduler: &mut Scheduler<Self>,
        ws: Websocket,
        tourn: Box<TournamentManager>,
    ) -> Watcher<()> {
        match self.cache.entry(tourn.id) {
            Entry::Occupied(mut entry) => match &mut entry.get_mut().comm {
                // Tournament is cached and communication is set up for it
                Some((_, broad)) => broad.subscribe(),
                // Tournament is cached but there is no communication for it
                None => {
                    let (sink, stream) = ws.split();
                    let (broad, sub) = watch_channel(());
                    entry.get_mut().comm = Some((sink, broad));
                    scheduler.add_stream(stream);
                    sub
                }
            },
            // Tournament is not cached
            Entry::Vacant(entry) => {
                let (sink, stream) = ws.split();
                let (broad, sub) = watch_channel(());
                let tc = TournComm {
                    tourn: *tourn,
                    comm: Some((sink, broad)),
                };
                let _ = entry.insert(tc);
                scheduler.add_stream(stream);
                sub
            }
        }
    }

    async fn handle_ws_msg(&mut self, scheduler: &mut Scheduler<Self>, msg: WebsocketMessage) {
        let WebsocketMessage::Bytes(data) = msg else {
            panic!("Server did not send bytes of Websocket")
        };
        let WebSocketMessage { body, id } =
            postcard::from_bytes::<ClientBoundMessage>(&data).unwrap();
        match body {
            ClientBound::FetchResp(_) => { /* Do nothing, handled elsewhere */ }
            ClientBound::SyncChain(link) => {
                self.handle_server_op_link(scheduler, &id, link).await;
            }
            ClientBound::SyncForward((t_id, sync)) => {
                self.handle_forwarded_sync(scheduler, &t_id, id, sync).await
            }
        }
    }

    fn handle_ws_err(&mut self, err: WebsocketError) {
        panic!("Got error from Websocket: {err:?}")
    }

    async fn handle_server_op_link(
        &mut self,
        scheduler: &mut Scheduler<Self>,
        msg_id: &Uuid,
        link: ServerOpLink,
    ) {
        // Get tourn
        let Some(t_id) = self.syncs.get_tourn_id(msg_id) else {
            return;
        };
        let Some(tourn) = self.cache.get_mut(&t_id) else {
            return;
        };
        match link {
            ServerOpLink::Conflict(proc) => {
                let server = ServerOpLink::Conflict(proc.clone());
                // TODO: This, somehow, needs to be a user decision...
                let dec: ClientOpLink = proc.purge().into();
                // Send decision to backend
                self.syncs.progress_chain(msg_id, dec.clone(), server);
                let msg = ServerBoundMessage {
                    id: *msg_id,
                    body: dec.into(),
                };
                tourn.send(scheduler, msg).await;
            }
            ServerOpLink::Completed(comp) => {
                tourn.tourn.handle_completion(comp).unwrap();
                self.syncs.finalize_chain(msg_id);
                (self.on_update)(t_id);
            }
            ServerOpLink::Error(_) | ServerOpLink::TerminatedSeen { .. } => {
                self.syncs.finalize_chain(msg_id);
            }
        }
    }

    async fn handle_forwarded_sync(
        &mut self,
        scheduler: &mut Scheduler<Self>,
        t_id: &TournamentId,
        msg_id: Uuid,
        sync: OpSync,
    ) {
        let Some(comm) = self.cache.get_mut(t_id) else {
            return;
        };
        let resp = if self.forwarded.contains_resp(&msg_id) {
            self.forwarded.get_resp(&msg_id).unwrap()
        } else {
            let resp = comm.tourn.handle_forwarded_sync(sync);
            if matches!(resp, SyncForwardResp::Success) {
                (self.on_update)(*t_id);
            }
            self.forwarded.add_resp(msg_id, resp.clone());
            resp
        };
        self.forwarded.clean();
        let msg = ServerBoundMessage {
            id: msg_id,
            body: resp.into(),
        };
        comm.send(scheduler, msg).await;
    }
}

async fn create_ws_connection(
    id: TournamentId,
    send: OneshotSender<Option<Watcher<()>>>,
) -> (
    Result<(Websocket, Box<TournamentManager>), ()>,
    OneshotSender<Option<Watcher<()>>>,
) {
    let url = format!(
        "ws{HOST_ADDRESS}{}",
        <Subscribe as GetRequest<1>>::ROUTE.replace([id.to_string().as_str()])
    );
    match Websocket::new(&url).await {
        Ok(mut ws) => {
            let msg = postcard::to_allocvec(&ServerBoundMessage::new(ServerBound::Fetch)).unwrap();
            ws.send(WebsocketMessage::Bytes(msg)).await.unwrap();
            let tourn = wait_for_tourn(&mut ws).await;
            (Ok((ws, tourn)), send)
        }
        Err(()) => (Err(()), send),
    }
}

async fn wait_for_tourn(stream: &mut Websocket) -> Box<TournamentManager> {
    loop {
        let Some(Ok(WebsocketMessage::Bytes(msg))) = stream.next().await else {
            continue;
        };
        let ClientBoundMessage { body, .. } = postcard::from_bytes(&msg).unwrap();
        let ClientBound::FetchResp(tourn) = body else {
            panic!("Server did not return a tournament")
        };
        return tourn;
    }
}

impl TournComm {
    async fn send(&mut self, scheduler: &mut Scheduler<ManagerState>, msg: ServerBoundMessage) {
        if let Some(comm) = self.comm.as_mut() {
            let bytes = WebsocketMessage::Bytes(postcard::to_allocvec(&msg).unwrap());
            let _ = comm.0.send(bytes).await;
            let retry = MessageRetry {
                msg,
                id: self.tourn.id,
            };
            scheduler.schedule(Instant::now() + RETRY_LIMIT, retry);
        }
    }
}

impl Trackable<TournamentManager, TournamentId> for ManagementCommand {
    fn track(tourn: TournamentManager) -> (Self, oneshot::Receiver<TournamentId>) {
        let (send, recv) = oneshot_channel();
        (Self::Import(Box::new(tourn), send), recv)
    }
}

impl Trackable<TournamentId, Option<Watcher<()>>> for ManagementCommand {
    fn track(id: TournamentId) -> (Self, oneshot::Receiver<Option<Watcher<()>>>) {
        let (send, recv) = oneshot_channel();
        (Self::Subscribe(id, send), recv)
    }
}

impl<F, T> Trackable<(TournamentId, F), Option<T>> for ManagementCommand
where
    F: 'static + Send + FnOnce(&TournamentManager) -> T,
    T: 'static + Send,
{
    fn track((id, query): (TournamentId, F)) -> (Self, oneshot::Receiver<Option<T>>) {
        let (send, recv) = oneshot_channel();
        let query = Box::new(move |tourn: Option<&TournamentManager>| {
            let _ = send.send(tourn.map(query));
        });
        (Self::Query(id, query), recv)
    }
}

impl Trackable<(TournamentId, UpdateType), Option<OpResult>> for ManagementCommand {
    fn track(
        (id, update): (TournamentId, UpdateType),
    ) -> (Self, oneshot::Receiver<Option<OpResult>>) {
        let (send, recv) = oneshot_channel();
        (Self::Update(id, update, send), recv)
    }
}

impl
    From<(
        Result<(Websocket, Box<TournamentManager>), ()>,
        OneshotSender<Option<Watcher<()>>>,
    )> for ManagementCommand
{
    fn from(
        (res, send): (
            Result<(Websocket, Box<TournamentManager>), ()>,
            OneshotSender<Option<Watcher<()>>>,
        ),
    ) -> Self {
        Self::Connection(res, send)
    }
}

impl From<WebsocketResult> for ManagementCommand {
    fn from(value: WebsocketResult) -> Self {
        ManagementCommand::Remote(value)
    }
}
impl From<MessageRetry> for ManagementCommand {
    fn from(value: MessageRetry) -> Self {
        Self::Retry(value)
    }
}

#[derive(Debug)]
pub(crate) struct MessageRetry {
    id: TournamentId,
    msg: ServerBoundMessage,
}

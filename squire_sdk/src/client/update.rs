use squire_lib::{
    operations::{OpResult, TournOp},
    tournament::TournamentId,
};
use tokio::sync::oneshot;

use super::error::ClientResult;

#[derive(Debug)]
pub(crate) struct TournamentUpdate {
    pub(crate) local: oneshot::Sender<Option<OpResult>>,
    pub(crate) remote: oneshot::Sender<Option<ClientResult<()>>>,
    pub(crate) id: TournamentId,
    pub(crate) update: UpdateType,
}

#[derive(Debug)]
pub struct UpdateTracker {
    local: oneshot::Receiver<Option<OpResult>>,
    remote: oneshot::Receiver<Option<ClientResult<()>>>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum UpdateType {
    Single(TournOp),
    Bulk(Vec<TournOp>),
}

pub(crate) fn update_channel(
    id: TournamentId,
    update: UpdateType,
) -> (TournamentUpdate, UpdateTracker) {
    let (local_send, local_recv) = oneshot::channel();
    let (remote_send, remote_recv) = oneshot::channel();
    let update = TournamentUpdate {
        local: local_send,
        remote: remote_send,
        id,
        update,
    };
    let tracker = UpdateTracker {
        local: local_recv,
        remote: remote_recv,
    };
    (update, tracker)
}

/*
pub enum UpdateStatus {
    Working,
    ChangedLocally(OpResult),
    PushedRemotely(ClientResult),
    Complete(OpResult, ClientResult),
}
*/

use squire_lib::{
    operations::{OpResult, TournOp},
    tournament::TournamentId,
};

use super::{
    compat::{oneshot, OneshotReceiver, OneshotSender},
    error::ClientResult,
};

#[derive(Debug)]
pub(crate) struct TournamentUpdate {
    pub(crate) local: OneshotSender<Option<OpResult>>,
    pub(crate) remote: OneshotSender<Option<ClientResult<()>>>,
    pub(crate) id: TournamentId,
    pub(crate) update: UpdateType,
}

#[derive(Debug)]
pub struct UpdateTracker {
    local: OneshotReceiver<Option<OpResult>>,
    remote: OneshotReceiver<Option<ClientResult<()>>>,
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
    let (local_send, local_recv) = oneshot();
    let (remote_send, remote_recv) = oneshot();
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

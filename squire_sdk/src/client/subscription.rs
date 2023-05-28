use squire_lib::{
    operations::{OpResult, TournOp},
    tournament::TournamentId,
};

use super::{
    compat::{oneshot, OneshotReceiver, OneshotSender, Subscriber},
    error::ClientResult,
};

/// Communicates two things:
///  - If there isn't one, open a Websocket connection for the specified tournament
///  - Return a broadcast subscriber that will communicate if a remote update has occured
/// The inner channel will send `None` if a Websocket connection could not be made
#[derive(Debug)]
pub(crate) struct TournamentSub {
    pub(crate) send: OneshotSender<Option<Subscriber<bool>>>,
    pub(crate) id: TournamentId,
}

#[derive(Debug)]
pub struct SubTracker {
    recv: OneshotReceiver<Option<Subscriber<bool>>>,
}

pub(crate) fn sub_channel(id: TournamentId) -> (TournamentSub, SubTracker) {
    let (send, recv) = oneshot();
    let update = TournamentSub { send, id };
    let tracker = SubTracker { recv };
    (update, tracker)
}

impl SubTracker {
    pub async fn process(self) -> Option<Subscriber<bool>> {
        self.recv.recv().await.flatten()
    }

    pub fn process_blocking(self) -> Option<Subscriber<bool>> {
        futures::executor::block_on(self.process())
    }
}

use std::{
    future::Future,
    pin::Pin,
    task::{self, Poll},
};

use squire_lib::tournament::TournamentId;
use tokio::sync::oneshot::{
    channel as oneshot, Receiver as OneshotReceiver, Sender as OneshotSender,
};

use crate::sync::TournamentManager;

#[derive(Debug)]
pub struct TournamentImport {
    pub(crate) tourn: TournamentManager,
    pub(crate) tracker: OneshotSender<TournamentId>,
}

#[derive(Debug)]
pub struct ImportTracker {
    tracker: OneshotReceiver<TournamentId>,
}

pub(crate) fn import_channel(tourn: TournamentManager) -> (TournamentImport, ImportTracker) {
    let (send, recv) = oneshot();
    let import = TournamentImport {
        tracker: send,
        tourn,
    };
    let tracker = ImportTracker { tracker: recv };
    (import, tracker)
}

impl ImportTracker {
    pub async fn process(self) -> Option<TournamentId> {
        self.tracker.await.ok()
    }

    pub fn process_blocking(self) -> Option<TournamentId> {
        futures::executor::block_on(self.process())
    }
}

impl Future for ImportTracker {
    type Output = Option<TournamentId>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.tracker).poll(cx).map(Result::ok)
    }
}

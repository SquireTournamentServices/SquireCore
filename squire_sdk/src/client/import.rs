use std::{
    future::Future,
    pin::Pin,
    task::{self, Poll},
    time::Duration,
};

use squire_lib::tournament::TournamentId;

use crate::tournaments::TournamentManager;

use super::compat::{oneshot, OneshotReceiver, OneshotSender, TryRecvError};

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
        self.tracker.recv().await
    }

    pub fn process_spin(mut self) -> Option<TournamentId> {
        loop {
            match self.tracker.try_recv() {
                Ok(val) => return Some(val),
                Err(TryRecvError::Disconnected) => return None,
                Err(TryRecvError::Empty) => {}
            }
        }
    }
}

impl Future for ImportTracker {
    type Output = Option<TournamentId>;

    fn poll(mut self: Pin<&mut Self>, _: &mut task::Context<'_>) -> Poll<Self::Output> {
        match self.tracker.try_recv() {
            Ok(val) => Poll::Ready(Some(val)),
            Err(TryRecvError::Disconnected) => Poll::Ready(None),
            Err(TryRecvError::Empty) => Poll::Pending,
        }
    }
}

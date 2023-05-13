use std::{
    future::Future,
    pin::Pin,
    task::{self, Poll},
    time::Duration,
};

use crate::tournaments::TournamentManager;

use super::compat::{oneshot, OneshotReceiver, OneshotSender};

#[derive(Debug)]
pub struct TournamentImport {
    pub(crate) tourn: TournamentManager,
    pub(crate) tracker: OneshotSender<()>,
}

#[derive(Debug)]
pub struct ImportTracker {
    tracker: OneshotReceiver<()>,
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
    pub async fn process(self) {
        let _ = self.tracker.recv().await;
    }

    pub fn process_spin(mut self) {
        while self.tracker.try_recv().is_err() {}
    }
}

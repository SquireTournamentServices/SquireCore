use tokio::sync::oneshot;

use crate::tournaments::TournamentManager;

#[derive(Debug)]
pub struct TournamentImport {
    pub(crate) tourn: TournamentManager,
    pub(crate) tracker: oneshot::Sender<()>,
}

#[derive(Debug)]
pub struct ImportTracker {
    tracker: oneshot::Receiver<()>,
}

pub(crate) fn import_channel(tourn: TournamentManager) -> (TournamentImport, ImportTracker) {
    let (send, recv) = oneshot::channel();
    let import = TournamentImport {
        tracker: send,
        tourn,
    };
    let tracker = ImportTracker {
        tracker: recv,
    };
    (import, tracker)
}

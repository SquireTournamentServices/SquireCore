use squire_sdk::{
    actor::*,
    api::SessionToken,
    model::identifiers::SquireAccountId,
    server::session::{AnyUser, SquireSession},
};
use tokio::sync::watch;

use super::SessionCommand;

impl Trackable<SquireAccountId, SessionToken> for SessionCommand {
    fn track(msg: SquireAccountId, send: OneshotSender<SessionToken>) -> Self {
        Self::Create(msg, send)
    }
}

impl Trackable<(), SessionToken> for SessionCommand {
    fn track((): (), send: OneshotSender<SessionToken>) -> Self {
        Self::Guest(send)
    }
}

impl Trackable<SessionToken, SquireSession> for SessionCommand {
    fn track(msg: SessionToken, send: OneshotSender<SquireSession>) -> Self {
        Self::Get(msg, send)
    }
}

impl Trackable<AnyUser, SessionToken> for SessionCommand {
    fn track(msg: AnyUser, send: OneshotSender<SessionToken>) -> Self {
        Self::Reauth(msg, send)
    }
}

impl Trackable<SessionToken, Option<watch::Receiver<SquireSession>>> for SessionCommand {
    fn track(msg: SessionToken, send: OneshotSender<Option<watch::Receiver<SquireSession>>>) -> Self {
        Self::Subscribe(msg, send)
    }
}

impl Trackable<AnyUser, bool> for SessionCommand {
    fn track(msg: AnyUser, send: OneshotSender<bool>) -> Self {
        Self::Delete(msg, send)
    }
}

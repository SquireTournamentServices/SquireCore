use std::fmt::Debug;

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
    fn track(
        msg: SessionToken,
        send: OneshotSender<Option<watch::Receiver<SquireSession>>>,
    ) -> Self {
        Self::Subscribe(msg, send)
    }
}

impl Trackable<AnyUser, bool> for SessionCommand {
    fn track(msg: AnyUser, send: OneshotSender<bool>) -> Self {
        Self::Delete(msg, send)
    }
}

impl Debug for SessionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Create(value, _) => write!(f, "Create({value:?})"),
            Self::Guest(_) => write!(f, "Guest()"),
            Self::Get(value, _) => write!(f, "Get({value:?})"),
            Self::Reauth(value, _) => write!(f, "Reauth({value:?})"),
            Self::Delete(value, _) => write!(f, "Delete({value:?})"),
            Self::Subscribe(value, _) => write!(f, "Subscribe({value:?})"),
            Self::Expiry(value) => write!(f, "Expiry({value:?})"),
            Self::Revoke(value) => write!(f, "Revoke({value:?})"),
        }
    }
}

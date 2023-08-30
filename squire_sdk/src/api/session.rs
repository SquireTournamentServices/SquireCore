use squire_lib::identifiers::SquireAccountId;

use super::SessionToken;

/// A user session for users that have an active session. Its primary usecase is for filtering
/// inbound websocket messages.
///
/// TODO: This type should also receive updates about the session so that such updates can be
/// communicated throughout the system.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum AuthUser {
    Guest(SessionToken),
    User(SquireAccountId),
}

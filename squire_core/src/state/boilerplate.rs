use std::fmt::Debug;

use super::SessionCommand;

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

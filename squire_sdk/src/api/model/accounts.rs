use serde::{Deserialize, Serialize};
use squire_lib::accounts::SquireAccount;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccount {
    account: SquireAccount,
}

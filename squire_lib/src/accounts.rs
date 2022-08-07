use identifiers::{UserAccountID, OrganizationAccountID};
use settings;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SharingPermissions {
    Everything,
    OnlyDeckList,
    OnlyDeckName,
    Nothing
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SquireAccount {
    display_name: String,
    user_name: String,
    gamer_tags = Vec<Option<String>>,
    user_id: UserAccountID,
    do_share: SharingPermissions,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OrganizationAccount {
    display_name: String,
    user_name: String,
    user_id: OrganizationAccountID,
    owner: SquireAccount,
    default_judge: Vec<SquireAccount>,
    admin_account: Vec<SquireAccount>,
    default_tournament_settings: settings,
}
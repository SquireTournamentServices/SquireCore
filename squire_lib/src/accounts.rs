use uuid::Uuid;
use mtgjson::model::deck::Deck;
use settings;

enum SharingPermissions {
    decklist{name: String, deck: Deck},
    user_name(String),
}

struct UserId {
    pub Uuid,
}

struct SquireAccount {
    display_name: String,
    user_name: String,
    user_id: UserId,
    do_share: SharingPermissions,
}

struct OrganizationAccount {
    display_name: String,
    user_name: String,
    user_id: UserId,
    owner: SquireAccount,
    default_judge: Vec<SquireAccount>,
    admin_account: Vec<SquireAccount>,
    default_tournament_settings: settings,
}
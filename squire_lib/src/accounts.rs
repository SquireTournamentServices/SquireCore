use uuid::Uuid;
use settings;

struct squire_account {
    display_name: String,
    user_name: String,
    user_id: Uuid,
    do_share: bool,
}

struct organization_account {
    display_name: String,
    user_name: String,
    user_id: Uuid,
    do_share: bool,
    owner: squire_account,
    default_judge: squire_account,
    admin_account: squire_account,
    default_tournament_settings: settings,
}
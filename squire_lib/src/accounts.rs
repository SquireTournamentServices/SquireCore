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
pub enum Platforms {
    Cockatrice,
    MTGOnline,
    Arena
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SquireAccount {
    pub display_name: String,
    pub user_name: String,
    pub gamer_tags: HashMap<Platforms, Option<String>>,
    pub user_id: UserAccountID,
    pub do_share: SharingPermissions,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OrganizationAccount {
    pub display_name: String,
    pub user_name: String,
    pub user_id: OrganizationAccountID,
    pub owner: SquireAccount,
    pub default_judge: Vec<SquireAccount>,
    pub admin_account: Vec<SquireAccount>,
    pub default_tournament_settings: settings,
}

impl SquireAccount {
    pub fn create(user_name: String, display_name: String, permissions: SharingPermissions) -> Self {
        SquireAccount {
            display_name,
            user_name,
            gamer_tags: HashMap::new(),
            user_id: UserAccountID::new(Uuid::new_v4()),
            permissions
        }
    }

    pub fn update_tags(&mut self, platfrom: Platforms, user_name: String) {
        self.gamer_tags.insert(platfrom, user_name);
    }

    pub fn get_tags(&self, platforms: Vec<Platforms>) -> Vec<Option<String>> {
        let tags = Vec::new()
        for platform in platforms {
            let gamer_tag = self.gamer_tags.get(platform);
            tags.insert(Some(gamer_tag.clone()));
        }
        tags
    }

    pub fn delete_tags(&mut self, platform: Platforms) {
        self.gamer_tags.insert(platform, None)
    }
}
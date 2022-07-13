use dashmap::DashMap;
use rocket::{get, routes};
use uuid::Uuid;
use squire_sdk::accounts::{AccountId, UserAccount, OrgAccount};

use squire_lib;

mod accounts;
mod tournament;

use accounts::{USERS_MAP, ORGS_MAP, users, all_users, orgs};

#[get("/world")]
fn world() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _ = USERS_MAP.set(DashMap::new());
    let _ = ORGS_MAP.set(DashMap::new());
    let id = AccountId(Uuid::new_v4());
    let account = UserAccount {
        external_id: id.clone(),
        display_name: "Tyler Bloom".to_string(),
        account_name: "TylerBloom".to_string(),
    };
    println!("{account:?}");
    USERS_MAP.get().unwrap().insert(id, account);
    let _rocket = rocket::build()
        .mount("/hello", routes![world])
        .mount("/accounts", routes![users, all_users, orgs])
        .launch()
        .await?;

    Ok(())
}

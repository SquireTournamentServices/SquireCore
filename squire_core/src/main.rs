use dashmap::DashMap;
use rocket::{get, routes};
use squire_sdk::accounts::{AccountId, UserAccount};
use uuid::Uuid;

mod accounts;
mod matches;
mod players;
mod tournaments;

use accounts::{all_users, orgs, users, ORGS_MAP, USERS_MAP};
use tournaments::{
    apply_tournament_op, create_tournament, get_all_tournaments, get_standings, get_tournament,
    TOURNS_MAP,
};

#[get("/world")]
fn world() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _ = USERS_MAP.set(DashMap::new());
    let _ = ORGS_MAP.set(DashMap::new());
    let _ = TOURNS_MAP.set(DashMap::new());
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
        .mount(
            "/tournaments",
            routes![
                create_tournament,
                apply_tournament_op,
                get_tournament,
                get_all_tournaments,
                get_standings
            ],
        )
        .launch()
        .await?;

    Ok(())
}

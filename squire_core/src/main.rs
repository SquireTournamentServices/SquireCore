use dashmap::DashMap;
use rocket::{get, routes};
use squire_sdk::accounts::{AccountId, UserAccount};
use uuid::Uuid;

mod accounts;
mod matches;
mod players;
mod tournaments;

use accounts::*;
use players::*;
use tournaments::*;

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
        .mount(
            "/tournaments/players",
            routes![
                get_player,
                get_all_players,
                get_active_players,
                get_player_count,
                get_active_player_count,
                get_player_deck,
                get_all_decks,
                get_all_player_decks,
                get_player_matches,
                get_latest_player_match,
            ],
        )
        .launch()
        .await?;

    Ok(())
}

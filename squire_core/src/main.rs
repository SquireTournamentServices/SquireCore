#![allow(unused)]

use dashmap::DashMap;
use mtgjson::mtgjson::atomics::Atomics;
use rocket::{data::DataStream, routes, Build, Rocket};
use std::time::Duration;
use tokio::sync::RwLock;
//use squire_sdk::accounts::{AccountId, UserAccount};
//use uuid::Uuid;

#[cfg(test)]
mod tests;

//mod accounts;
mod cards;
mod matches;
mod players;
mod tournaments;
mod accounts;

//use accounts::*;
use cards::*;
use players::*;
use tournaments::*;

pub fn init() -> Rocket<Build> {
    //let _ = USERS_MAP.set(DashMap::new());
    //let _ = ORGS_MAP.set(DashMap::new());
    let _ = TOURNS_MAP.set(DashMap::new());
    rocket::build()
        //.mount("/api/v1/accounts", routes![users, all_users, orgs])
        .mount(
            "/api/v1/tournaments",
            routes![
                create_tournament,
                get_tournament,
                get_all_tournaments,
                get_standings,
                slice_ops,
                sync,
                rollback,
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
        .mount("/api/v1/cards", routes![atomics, meta])
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let client = init();
    /*
    let id = AccountId(Uuid::new_v4());
    let account = UserAccount {
        external_id: id.clone(),
        display_name: "Tyler Bloom".to_string(),
        account_name: "TylerBloom".to_string(),
    };
    println!("{account:?}");
    USERS_MAP.get().unwrap().insert(id, account);
    */
    // Spawns an await task to update the card collection every week.
    //MINIMAL_CACHE.set(DashMap::new()).unwrap();
    let atomics: Atomics = reqwest::get("https://mtgjson.com/api/v5/AtomicCards.json")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    ATOMICS_MAP.set(RwLock::new(atomics)).unwrap();
    let meta: MetaChecker = reqwest::get("https://mtgjson.com/api/v5/Meta.json")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    META_CACHE.set(RwLock::new(meta.meta)).unwrap();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1800));
        interval.tick().await;
        loop {
            interval.tick().await;
            let meta_data: MetaChecker =
                if let Ok(data) = reqwest::get("https://mtgjson.com/api/v5/Meta.json").await {
                    if let Ok(data) = data.json().await {
                        data
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };
            let meta = META_CACHE.get().unwrap().read().await;
            if meta_data.meta == *meta {
                continue;
            }
            let atomics: Atomics = if let Ok(data) =
                reqwest::get("https://mtgjson.com/api/v5/AtomicCards.json").await
            {
                if let Ok(data) = data.json().await {
                    data
                } else {
                    continue;
                }
            } else {
                continue;
            };
            let mut cards = ATOMICS_MAP.get().unwrap().write().await;
            *cards = atomics;
            let mut meta = META_CACHE.get().unwrap().write().await;
            *meta = meta_data.meta;
        }
    });
    client.launch().await?;

    Ok(())
}

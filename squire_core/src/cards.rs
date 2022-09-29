use dashmap::DashMap;
use mtgjson::mtgjson::{atomics::Atomics, meta::Meta};
use once_cell::sync::OnceCell;
use rocket::{get, post, serde::json::Json};

use cycle_map::cycle_map::CycleMap;
use serde::{Deserialize, Serialize};
use squire_sdk::cards::{AtomicCardsResponse, MetaResponse};
use tokio::sync::RwLock;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// Used for requests to MTGJSON
pub struct MetaChecker {
    pub meta: Meta,
    pub data: Meta,
}

/// The meta data from the last time the atomics collection was built
pub static META_CACHE: OnceCell<RwLock<Meta>> = OnceCell::new();
/// The latest collection of atomic cards
pub static ATOMICS_MAP: OnceCell<RwLock<Atomics>> = OnceCell::new();
///// The cache of collections of minimal cards
//pub static MINIMAL_CACHE: OnceCell<DashMap<String, MinimalCardCollection>> = OnceCell::new();

#[get("/meta")]
pub async fn meta() -> MetaResponse {
    let meta: Meta = META_CACHE.get().unwrap().read().await.clone();
    MetaResponse::new(meta)
}

#[get("/atomics")]
pub async fn atomics() -> AtomicCardsResponse {
    let meta: Meta = META_CACHE.get().unwrap().read().await.clone();
    let atomics: Atomics = ATOMICS_MAP.get().unwrap().read().await.clone();
    AtomicCardsResponse::new((meta, atomics))
}

/*
#[get("/minimal/<lang>")]
pub async fn minimal(lang: String) -> MinimalCardsResponse {
    let meta: Meta = META_CACHE.get().unwrap().read().await.clone();
    match MINIMAL_CACHE.get().unwrap().get(&lang) {
        Some(cards) => MinimalCardsResponse::new((meta, cards.clone())),
        None => {
            let atomics = ATOMICS_MAP.get().unwrap().read().await;
            let cards: CycleMap<String, MinimalCard> = atomics
                .data
                .iter()
                .map(|(name, card)| {
                    (
                        name.clone(),
                        card.as_minimal(&lang)
                            .expect(&format!("Could not minimize {name} in {lang}")),
                    )
                })
                .collect();
            let coll = MinimalCardCollection { cards };
            MINIMAL_CACHE.get().unwrap().insert(lang, coll.clone());
            MinimalCardsResponse::new((meta, coll))
        }
    }
}
*/

use dashmap::DashMap;
use once_cell::sync::OnceCell;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use cycle_map::cycle_map::CycleMap;

use squire_sdk::cards::{atomics::Atomics, meta::Meta, AtomicCardsResponse, MetaResponse};

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

pub async fn meta() -> MetaResponse {
    let meta: Meta = META_CACHE.get().unwrap().read().await.clone();
    MetaResponse::new(meta)
}

pub async fn atomics() -> AtomicCardsResponse {
    let meta: Meta = META_CACHE.get().unwrap().read().await.clone();
    let atomics: Atomics = ATOMICS_MAP.get().unwrap().read().await.clone();
    AtomicCardsResponse::new((meta, atomics))
}

pub async fn update_cards() {
    let meta_data: MetaChecker =
        if let Ok(data) = reqwest::get("https://mtgjson.com/api/v5/Meta.json").await {
            if let Ok(data) = data.json().await {
                data
            } else {
                return;
            }
        } else {
            return;
        };
    let meta = META_CACHE.get().unwrap().read().await;
    if meta_data.meta == *meta {
        return;
    }
    let atomics: Atomics =
        if let Ok(data) = reqwest::get("https://mtgjson.com/api/v5/AtomicCards.json").await {
            if let Ok(data) = data.json().await {
                data
            } else {
                return;
            }
        } else {
            return;
        };
    let mut cards = ATOMICS_MAP.get().unwrap().write().await;
    *cards = atomics;
    let mut meta = META_CACHE.get().unwrap().write().await;
    *meta = meta_data.meta;
}

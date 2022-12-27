use std::{error::Error, time::Duration};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::cards::{atomics::Atomics, meta::Meta, AtomicCardsResponse, MetaResponse};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// Used for requests to MTGJSON
pub struct MetaChecker {
    pub meta: Meta,
    pub data: Meta,
}

pub async fn meta() -> MetaResponse {
    let meta: Meta = META_CACHE.get().unwrap().read().await.clone();
    MetaResponse::new(meta)
}

pub async fn atomics() -> AtomicCardsResponse {
    let meta: Meta = META_CACHE.get().unwrap().read().await.clone();
    let atomics: Atomics = ATOMICS_MAP.get().unwrap().read().await.clone();
    AtomicCardsResponse::new((meta, atomics))
}

async fn update_cards() -> Result<(), Box<dyn Error>> {
    let meta_data: MetaChecker = reqwest::get("https://mtgjson.com/api/v5/Meta.json").await?.json().await?;
    if meta_data.meta == *meta {
        return;
    }
    let atomics: Atomics = reqwest::get("https://mtgjson.com/api/v5/AtomicCards.json").await?.json().await?;
    let mut cards = ATOMICS_MAP.get().unwrap().write().await;
    *cards = atomics;
    let mut meta = META_CACHE.get().unwrap().write().await;
    *meta = meta_data.meta;
}

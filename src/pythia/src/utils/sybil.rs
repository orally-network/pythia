use anyhow::{anyhow, Result};
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::{
    clone_with_state, log, metrics, types::{logger::SYBIL, asset_data::AssetDataResult}, STATE,
};

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
enum FeedDataResponse {
    Ok(AssetDataResult),
    Err(String),
}

pub async fn is_feed_exists(feed_id: &str) -> Result<bool> {
    let sybil_canister =
        clone_with_state!(sybil_canister).expect("SYBIL CANISTER should be initialised");

    let feed_id = feed_id.to_string();

    metrics!(inc SYBIL_OUTCALLS, "is_feed_exists");
    let (is_exist,): (bool,) = ic_cdk::call(sybil_canister, "is_feed_exists", (feed_id,))
        .await
        .map_err(|(code, msg)| anyhow!("{:?}: {}", code, msg))?;

    metrics!(inc SUCCESSFUL_SYBIL_OUTCALLS, "is_feed_exists");

    Ok(is_exist)
}

pub async fn get_asset_data(feed_id: &str) -> Result<AssetDataResult> {
    log!("[{SYBIL}] Asset data requested for feed_id: {:#?}", feed_id);
    let sybil_canister = STATE.with(|state| {
        state
            .borrow()
            .sybil_canister
            .expect("SYBIL CANISTER should be initialised")
    });

    let feed_id = feed_id.to_string();
    log!("Preparing to call");

    metrics!(inc SYBIL_OUTCALLS, "get_asset_data");
    let (feed_data,): (Result<AssetDataResult, String>,) =
        ic_cdk::call(sybil_canister, "get_asset_data", (feed_id,))
            .await
            .map_err(|(code, msg)| anyhow!("{:?}: {}", code, msg))?;
    metrics!(inc SUCCESSFUL_SYBIL_OUTCALLS, "get_asset_data");

    log!("Call returned");

    match feed_data {
        Result::Ok(data) => Ok(data),
        Result::Err(err) => Err(anyhow!(err)),
    }
}

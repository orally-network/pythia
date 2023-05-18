use anyhow::{anyhow, Result};

use ic_cdk::export::{
    candid::CandidType,
    serde::{Deserialize, Serialize},
};

use crate::{types::rate_data::RateDataLight, SYBIL_CANISTER};

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
enum PairDataResponse {
    Ok(RateDataLight),
    Err(String),
}

pub async fn is_pair_exists(pair_id: &str) -> Result<bool> {
    let sybil_canister = SYBIL_CANISTER.with(|sybil_canister| {
        sybil_canister
            .borrow()
            .expect("SYBIL CANISTER should be initialised")
    });

    let pair_id = pair_id.to_string();
    let (is_exist,): (bool,) = ic_cdk::call(sybil_canister, "is_pair_exists", (pair_id,))
        .await
        .map_err(|(code, msg)| anyhow!("{:?}: {}", code, msg))?;

    Ok(is_exist)
}

pub async fn get_asset_data(pair_id: &str) -> Result<RateDataLight> {
    let sybil_canister = SYBIL_CANISTER.with(|sybil_canister| {
        sybil_canister
            .borrow()
            .expect("SYBIL CANISTER should be initialised")
    });

    let pair_id = pair_id.to_string();
    let (pair_data,): (PairDataResponse,) =
        ic_cdk::call(sybil_canister, "get_asset_data", (pair_id,))
            .await
            .map_err(|(code, msg)| anyhow!("{:?}: {}", code, msg))?;

    match pair_data {
        PairDataResponse::Ok(data) => Ok(data),
        PairDataResponse::Err(err) => Err(anyhow!(err)),
    }
}

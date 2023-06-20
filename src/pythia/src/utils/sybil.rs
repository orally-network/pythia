use anyhow::{anyhow, Result};

use ic_cdk::export::{
    candid::CandidType,
    serde::{Deserialize, Serialize},
};

use crate::{clone_with_state, types::rate_data::RateDataLight, STATE};

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
enum PairDataResponse {
    Ok(RateDataLight),
    Err(String),
}

pub async fn is_pair_exists(pair_id: &str) -> Result<bool> {
    let sybil_canister =
        clone_with_state!(sybil_canister).expect("SYBIL CANISTER should be initialised");

    let pair_id = pair_id.to_string();
    let (is_exist,): (bool,) = ic_cdk::call(sybil_canister, "is_pair_exists", (pair_id,))
        .await
        .map_err(|(code, msg)| anyhow!("{:?}: {}", code, msg))?;

    Ok(is_exist)
}

pub async fn get_asset_data(pair_id: &str) -> Result<RateDataLight> {
    let sybil_canister = STATE.with(|state| {
        state
            .borrow()
            .sybil_canister
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

use std::str::FromStr;

use anyhow::{Result, anyhow};

use ic_cdk::{export::Principal, api::management_canister::{provisional::CanisterIdRecord, main::canister_status}};
use ic_web3::{ic::get_eth_addr, types::H160};

use crate::{clone_with_state, update_state, utils::{address, canister}};

pub async fn get_controllers() -> Result<Vec<Principal>> {
    let (canister_status,) = canister_status(CanisterIdRecord {
        canister_id: ic_cdk::id(),
    })
        .await
        .map_err(|(rej_code, msg)| anyhow!("canister_status rejected with code: {:?}, msg: {:?}", rej_code, msg))?;

    Ok(canister_status.settings.controllers)
}

pub async fn pma() -> Result<String> {
    if let Some(pma) = clone_with_state!(pma) {
        return Ok(pma);
    }

    let addr = get_eth_addr(None, Some(vec![vec![]]), clone_with_state!(key_name))
        .await
        .map(|addr| address::from_h160(&addr))
        .map_err(|e| anyhow!("{e}"))?;

    update_state!(pma, Some(addr.clone()));
    Ok(addr)
}

pub async fn pma_h160() -> Result<H160> {
    Ok(H160::from_str(&canister::pma().await?).expect("pma should be a valid address"))
}
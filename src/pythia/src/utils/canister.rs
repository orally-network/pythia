use std::str::FromStr;

use anyhow::{anyhow, Context, Result};

use candid::Nat;
use ic_cdk::{
    api::management_canister::{main::canister_status, provisional::CanisterIdRecord},
    export::Principal,
};
use ic_web3::{ic::get_eth_addr, types::H160};

use crate::{
    clone_with_state,
    types::{balance::Balances, chains::Chains, errors::PythiaError},
    update_state,
    utils::{address, canister, sybil},
};

const DECIMALS: &str = "1000000000000000000";
const FEE_IN_USDT: &str = "67500";

pub async fn get_controllers() -> Result<Vec<Principal>> {
    let (canister_status,) = canister_status(CanisterIdRecord {
        canister_id: ic_cdk::id(),
    })
    .await
    .map_err(|(rej_code, msg)| {
        anyhow!(
            "canister_status rejected with code: {:?}, msg: {:?}",
            rej_code,
            msg
        )
    })?;

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

pub async fn fee(chain_id: &Nat) -> Result<Nat> {
    let mut pair_id = Chains::get(chain_id)?.symbol;
    pair_id.push_str("/USDT");

    if sybil::is_pair_exists(&pair_id).await? {
        let rate = sybil::get_asset_data(&pair_id)
            .await
            .context(PythiaError::UnableToGetAssetData)?;
        let decimals = Nat::from_str(DECIMALS).context(PythiaError::InvalidNumber)?;
        let fee_in_usdt = Nat::from_str(FEE_IN_USDT).context(PythiaError::InvalidNumber)?;

        return Ok((fee_in_usdt * decimals) / rate.rate);
    }

    Chains::get_fee(chain_id)
}

pub fn collect_fee(chain_id: &Nat, receiver: &str, amount: &Nat) -> Result<()> {
    Balances::add_amount(chain_id, receiver, amount).context(PythiaError::UnableToIncreaseBalance)
}

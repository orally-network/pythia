use std::str::FromStr;

use anyhow::{anyhow, Context, Result};

use candid::Nat;
use ic_cdk::api::management_canister::http_request::{TransformContext, TransformFunc};
use ic_web3_rs::{
    ic::get_eth_addr,
    transports::ic_http_client::{CallOptions, CallOptionsBuilder},
    types::H160,
};

use crate::{
    clone_with_state, log,
    types::{balance::Balances, chains::Chains, errors::PythiaError},
    update_state,
    utils::{address, canister, sybil},
};

const DECIMALS: &str = "1000000000000000000";

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
    log!("Trying to get fee for chain_id: {}", chain_id);
    let mut pair_id = Chains::get_symbol(chain_id)?;
    pair_id.push_str("/USD");

    if sybil::is_pair_exists(&pair_id).await? {
        log!("Pair exists in Sybild");
        let rate = sybil::get_asset_data(&pair_id)
            .await
            .context(PythiaError::UnableToGetAssetData)?;
        let decimals = Nat::from_str(DECIMALS)?;
        let fee_in_usdt = clone_with_state!(tx_fee);

        log!("Returning fee from Sybil");
        return Ok((fee_in_usdt * decimals) / rate.rate);
    }

    log!("Returning fee from State");
    Chains::get_fee(chain_id)
}

pub fn collect_fee(chain_id: &Nat, receiver: &str, amount: &Nat) -> Result<()> {
    Balances::add_amount(chain_id, receiver, amount).context(PythiaError::UnableToIncreaseBalance)
}

pub fn transform_ctx_tx() -> CallOptions {
    get_transform_ctx("transform_tx")
}

pub fn transform_ctx_tx_with_logs() -> CallOptions {
    get_transform_ctx("transform_tx_with_logs")
}

pub fn transform_ctx() -> CallOptions {
    get_transform_ctx("transform")
}

fn get_transform_ctx(method: &str) -> CallOptions {
    CallOptionsBuilder::default()
        .transform(Some(TransformContext {
            function: TransformFunc(candid::Func {
                principal: ic_cdk::api::id(),
                method: method.into(),
            }),
            context: vec![],
        }))
        .cycles(None)
        .max_resp(None)
        .build()
        .expect("failed to build call options")
}

pub fn set_custom_panic_hook() {
    _ = std::panic::take_hook(); // clear custom panic hook and set default
    let old_handler = std::panic::take_hook(); // take default panic hook

    // set custom panic hook
    std::panic::set_hook(Box::new(move |info| {
        log!("PANIC OCCURRED: {:#?}", info);
        old_handler(info);
    }));
}

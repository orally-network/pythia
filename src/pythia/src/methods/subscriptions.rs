use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};

use ic_cdk::export::candid::Nat;
use ic_cdk_macros::{query, update};
use ic_utils::logger::log_message;

use crate::{
    clone_with_state,
    jobs::publisher,
    types::{subscription::Subscription, whitelist},
    utils::{check_balance, check_subs_limit, get_chain, rec_eth_addr, validate_caller},
    PythiaError, STATE,
};

#[update]
#[allow(clippy::too_many_arguments)]
pub async fn subscribe(
    chain_id: Nat,
    pair_id: Option<String>,
    contract_addr: String,
    method_abi: String,
    frequency: Nat,
    is_random: bool,
    gas_limit: Nat,
    msg: String,
    sig: String,
) -> Result<(), String> {
    _subscribe(
        chain_id,
        pair_id,
        contract_addr,
        method_abi,
        frequency,
        is_random,
        gas_limit,
        msg,
        sig,
    )
    .await
    .map_err(|e| format!("{e:?}"))
}

#[allow(clippy::too_many_arguments)]
async fn _subscribe(
    chain_id: Nat,
    pair_id: Option<String>,
    contract_addr: String,
    method_abi: String,
    frequency: Nat,
    is_random: bool,
    gas_limit: Nat,
    msg: String,
    sig: String,
) -> Result<()> {
    if frequency < clone_with_state!(timer_frequency) {
        return Err(anyhow!("frequency must be greater than 5 minutes"));
    };

    let chain = get_chain(&chain_id)?;
    let pub_key = rec_eth_addr(&msg, &sig).await?;

    check_subs_limit(&pub_key)?;
    if !check_balance(&pub_key, &chain) {
        return Err(anyhow!("not enough balance"));
    }

    let subscription = Subscription::builder()
        .owner(&hex::encode(pub_key.as_bytes()))
        .contract(&contract_addr)
        .pair(pair_id)
        .method(&method_abi, &gas_limit, &frequency)
        .random(is_random)
        .build()
        .await
        .context("failed to create a subscription")?;

    log_message(format!(
        "[ADDR: 0x{}] subsciption creation; sub id: {}",
        hex::encode(pub_key.as_bytes()),
        subscription.id
    ));

    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state
            .subscriptions
            .entry(chain_id)
            .or_insert(vec![])
            .push(subscription);
    });

    if !clone_with_state!(is_timer_active) {
        publisher::execute();
    }

    Ok(())
}

#[query]
pub fn get_subscriptions(pub_key: Option<String>) -> Vec<Subscription> {
    STATE.with(|state| {
        state
            .borrow()
            .subscriptions
            .values()
            .map(|subs| {
                subs.iter()
                    .filter(|sub| {
                        if let Some(pub_key) = &pub_key {
                            sub.owner == *pub_key
                        } else {
                            true
                        }
                    })
                    .cloned()
                    .collect::<Vec<Subscription>>()
            })
            .fold(vec![], |result, v| {
                let mut result = result;
                result.extend(v);
                result
            })
    })
}

#[update]
pub async fn stop_sub(chain_id: Nat, sub_id: Nat, msg: String, sig: String) -> Result<(), String> {
    _stop_sub(chain_id, sub_id, msg, sig)
        .await
        .map_err(|e| format!("{e:?}"))
}

pub async fn _stop_sub(chain_id: Nat, sub_id: Nat, msg: String, sig: String) -> Result<()> {
    let pub_key = rec_eth_addr(&msg, &sig).await?;
    let owner = hex::encode(pub_key.as_bytes());
    if !whitelist::is_whitelisted(&owner) {
        return Err(anyhow!("not whitelisted"));
    }

    STATE.with(|state| {
        state
            .borrow_mut()
            .subscriptions
            .get_mut(&chain_id)
            .ok_or(PythiaError::ChainDoesNotExist)?
            .iter_mut()
            .find(|s| s.id == sub_id)
            .ok_or(PythiaError::SubDoesNotExist)?
            .status
            .is_active = false;

        log_message(format!("[USER: {}] stop sub_id: {}", owner, sub_id));
        Ok(())
    })
}

#[update]
pub async fn start_sub(chain_id: Nat, sub_id: Nat, msg: String, sig: String) -> Result<(), String> {
    _start_sub(chain_id, sub_id, msg, sig)
        .await
        .map_err(|e| format!("{e:?}"))
}

pub async fn _start_sub(chain_id: Nat, sub_id: Nat, msg: String, sig: String) -> Result<()> {
    let pub_key = rec_eth_addr(&msg, &sig).await?;
    let owner = hex::encode(pub_key.as_bytes());
    if !whitelist::is_whitelisted(&owner) {
        return Err(anyhow!("not whitelisted"));
    }

    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state
            .subscriptions
            .get_mut(&chain_id)
            .ok_or(PythiaError::ChainDoesNotExist)?
            .iter_mut()
            .find(|s| s.id == sub_id)
            .ok_or(PythiaError::SubDoesNotExist)?
            .status
            .is_active = true;

        log_message(format!("[USER: {}] start sub_id: {}", owner, sub_id));

        if !state.is_timer_active {
            publisher::execute();
        }

        Ok(())
    })
}

#[update]
pub async fn update_sub_gas_limit(
    gas_limit: Nat,
    chain_id: Nat,
    sub_id: Nat,
    msg: String,
    sig: String,
) -> Result<(), String> {
    _update_sub_gas_limit(gas_limit, chain_id, sub_id, msg, sig)
        .await
        .map_err(|e| format!("{e:?}"))
}

pub async fn _update_sub_gas_limit(
    gas_limit: Nat,
    chain_id: Nat,
    sub_id: Nat,
    msg: String,
    sig: String,
) -> Result<()> {
    let pub_key = rec_eth_addr(&msg, &sig).await?;
    let owner = hex::encode(pub_key.as_bytes());
    if !whitelist::is_whitelisted(&owner) {
        return Err(anyhow!("not whitelisted"));
    }

    STATE.with(|state| {
        state
            .borrow_mut()
            .subscriptions
            .get_mut(&chain_id)
            .ok_or(PythiaError::ChainDoesNotExist)?
            .iter_mut()
            .find(|s| s.id == sub_id)
            .ok_or(PythiaError::SubDoesNotExist)?
            .method
            .gas_limit = gas_limit;

        log_message(format!(
            "[USER: {}] update gas_limit, sub_id: {}",
            owner, sub_id
        ));
        Ok(())
    })
}

#[update]
pub fn stop_subs() -> Result<(), String> {
    _stop_subs().map_err(|e| format!("{e:?}"))
}

fn _stop_subs() -> Result<()> {
    validate_caller()?;

    log_message("subsctiptions stopped".into());

    STATE.with(|state| {
        state
            .borrow_mut()
            .subscriptions
            .iter_mut()
            .for_each(|(_, subscriptions)| {
                subscriptions.iter_mut().for_each(|subscription| {
                    subscription.status.is_active = false;
                })
            });

        Ok(())
    })
}

#[update]
pub fn remove_subs() -> Result<(), String> {
    _remove_subs().map_err(|e| format!("{e:?}"))
}

pub fn _remove_subs() -> Result<()> {
    validate_caller()?;

    STATE.with(|s| s.borrow_mut().subscriptions = HashMap::default());

    log_message("subsctiptions removed".into());

    Ok(())
}

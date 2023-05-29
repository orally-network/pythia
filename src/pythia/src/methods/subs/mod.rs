use std::{str::FromStr, time::Duration};

use anyhow::{anyhow, Context, Result};

use ic_cdk::export::candid::Nat;
use ic_cdk_macros::{query, update};
use ic_cdk_timers::{clear_timer, set_timer_interval, TimerId};
use ic_utils::logger::log_message;
use ic_web3::types::H160;

use crate::{
    utils::{check_balance, collect_fee, publish::publish, rec_eth_addr, validate_caller},
    CandidSub, Chain, PythiaError, Sub, CHAINS, SUBS, SUBS_LIMIT_TOTAL, SUBS_LIMIT_WALLET, U256,
};

use super::get_exec_addr_from_pub;

const FREQUENCY_LIMIT: u64 = 5 * 60;

#[update]
#[allow(clippy::too_many_arguments)]
pub async fn subscribe(
    chain_id: Nat,
    pair_id: Option<String>,
    contract_addr: String,
    method_abi: String,
    frequency: Nat,
    is_random: bool,
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
    msg: String,
    sig: String,
) -> Result<()> {
    let frequency = *frequency
        .0
        .to_u64_digits()
        .last()
        .expect("frequency should be u64");

    if frequency < FREQUENCY_LIMIT {
        return Err(anyhow!("frequency must be greater than 5 minutes"));
    }

    let chain_id = U256::from(chain_id);
    let chain = get_chain(&chain_id)?;
    let pub_key = rec_eth_addr(&msg, &sig).await?;
    check_subs_limit_total(&pub_key)?;

    let exec_addr = get_exec_addr_from_pub(&pub_key).await?;
    check_balance(&exec_addr, &chain).await?;
    collect_fee(&pub_key, &exec_addr, &chain).await?;

    let sub = Sub::instance(
        &chain,
        pair_id,
        &contract_addr,
        &method_abi,
        &frequency,
        &pub_key,
        &exec_addr,
        is_random,
    )
    .await?;
    add_sub(&sub);

    log_message(format!(
        "[ADDR: 0x{}] sub creation; sub id: {}",
        hex::encode(pub_key.as_bytes()),
        sub.id
    ));

    Ok(())
}

fn check_subs_limit_total(pub_key: &H160) -> Result<()> {
    SUBS.with(|subs| {
        let subs = subs.borrow();

        if subs.len() as u64 >= SUBS_LIMIT_TOTAL.with(|s| *s.borrow()) {
            return Err(anyhow!("Subs total limit reached"));
        }

        let mut subs_counter = 0;
        for sub in subs.iter() {
            if sub.owner == *pub_key {
                subs_counter += 1;
            }
        }

        if subs_counter >= SUBS_LIMIT_WALLET.with(|s| *s.borrow()) {
            return Err(anyhow!("Subs limit per waller reached"));
        }

        Ok(())
    })
}

#[query]
pub fn get_subs(pub_key: String) -> Result<Vec<CandidSub>, String> {
    _get_subs(pub_key).map_err(|e| format!("{e:?}"))
}

fn _get_subs(pub_key: String) -> Result<Vec<CandidSub>> {
    let pub_key = H160::from_str(&pub_key)?;

    let subs: Vec<CandidSub> = SUBS.with(|subs| {
        subs.borrow()
            .iter()
            .filter(|s| s.owner == pub_key)
            .map(|s| s.clone().into())
            .collect()
    });

    Ok(subs)
}

#[update]
pub async fn refresh_subs(chain_id: Nat, msg: String, sig: String) -> Result<(), String> {
    _refresh_subs(chain_id, msg, sig)
        .await
        .map_err(|e| format!("{e:?}"))
}

async fn _refresh_subs(chain_id: Nat, msg: String, sig: String) -> Result<()> {
    let chain_id = U256::from(chain_id);
    let chain = get_chain(&chain_id)?;
    let pub_key = rec_eth_addr(&msg, &sig).await?;
    let exec_addr = get_exec_addr_from_pub(&pub_key).await?;

    check_balance(&exec_addr, &chain).await?;

    SUBS.with(|subs| {
        let mut subs = subs.borrow_mut();

        for sub in subs.iter_mut() {
            if sub.owner == pub_key {
                let id = sub.id;

                let timer_id = set_timer_interval(Duration::from_secs(sub.frequency), move || {
                    publish(id, pub_key);
                });

                sub.timer_id = serde_json::to_string(&timer_id).expect("should be valid timer id");
                sub.is_active = true;
            }
        }

        log_message(format!(
            "[USER: {}] subs refresh; chain id: {}",
            hex::encode(pub_key.as_bytes()),
            chain_id.0
        ));

        Ok(())
    })
}

pub fn get_chain(chain_id: &U256) -> Result<Chain> {
    CHAINS.with(|chains| {
        Ok(chains
            .borrow()
            .get(chain_id)
            .ok_or(PythiaError::ChainDoesNotExist)?
            .clone())
    })
}

pub fn add_sub(sub: &Sub) {
    SUBS.with(|subs| subs.borrow_mut().push(sub.clone()))
}

#[update]
pub async fn stop_sub(sub_id: Nat, msg: String, sig: String) -> Result<(), String> {
    _stop_sub(sub_id, msg, sig)
        .await
        .map_err(|e| format!("{e:?}"))
}

pub async fn _stop_sub(sub_id: Nat, msg: String, sig: String) -> Result<()> {
    let pub_key = rec_eth_addr(&msg, &sig).await?;

    let sub_id_digits = sub_id.0.to_u64_digits();
    let mut sub_id: u64 = 0;
    if !sub_id_digits.is_empty() {
        sub_id = *sub_id_digits.last().expect("sub_id should be a number");
    }

    SUBS.with(|subs| {
        let mut subs = subs.borrow_mut();

        let mut sub = subs
            .get_mut(sub_id as usize)
            .context("Sub does not exist")?;

        let timer_id: TimerId =
            serde_json::from_str(&sub.timer_id).expect("should be valid timer id");

        clear_timer(timer_id);

        sub.is_active = false;

        log_message(format!(
            "[USER: {}] stop sub_id: {}",
            hex::encode(pub_key.as_bytes()),
            sub_id
        ));
        Ok(())
    })
}

#[update]
pub async fn start_sub(sub_id: Nat, msg: String, sig: String) -> Result<(), String> {
    _start_sub(sub_id, msg, sig)
        .await
        .map_err(|e| format!("{e:?}"))
}

pub async fn _start_sub(sub_id: Nat, msg: String, sig: String) -> Result<()> {
    let pub_key = rec_eth_addr(&msg, &sig).await?;

    let sub_id_digits = sub_id.0.to_u64_digits();
    let mut sub_id: u64 = 0;
    if !sub_id_digits.is_empty() {
        sub_id = *sub_id_digits.last().expect("sub_id should be a number");
    }

    SUBS.with(|subs| {
        let mut subs = subs.borrow_mut();

        let mut sub = subs
            .get_mut(sub_id as usize)
            .context("Sub does not exist")?;

        let id = sub.id;

        let timer_id = set_timer_interval(Duration::from_secs(sub.frequency), move || {
            publish(id, pub_key);
        });

        sub.timer_id = serde_json::to_string(&timer_id).expect("should be valid timer id");
        sub.is_active = true;

        log_message(format!(
            "[USER: {}] start sub_id: {}",
            hex::encode(pub_key.as_bytes()),
            sub_id
        ));

        Ok(())
    })
}

#[update]
pub fn stop_subs() -> Result<(), String> {
    _stop_subs()
        .map_err(|e| format!("{e:?}"))
}

fn _stop_subs() -> Result<()> {
    validate_caller()?;

    SUBS.with(|subs| {
        let mut subs = subs.borrow_mut();
        for mut sub in subs.iter_mut() {
            let timer_id: TimerId =
                serde_json::from_str(&sub.timer_id).expect("should be valid timer id");

            clear_timer(timer_id);

            sub.is_active = false;
        }

        Ok(())
    })
}

#[update]
pub fn remove_subs() -> Result<(), String> {
    _remove_subs()
        .map_err(|e| format!("{e:?}"))
}

pub fn _remove_subs() -> Result<()> {
    validate_caller()?;

    SUBS.with(|subs| {
        let subs = subs.borrow();
        for sub in subs.iter() {
            let timer_id: TimerId =
                serde_json::from_str(&sub.timer_id).expect("should be valid timer id");

            clear_timer(timer_id);
        }
    });

    SUBS.with(|s| s.replace(vec![]));

    Ok(())
}
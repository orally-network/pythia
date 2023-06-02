use std::time::Duration;

use anyhow::{anyhow, Context, Result};

use ic_cdk::export::candid::Nat;
use ic_cdk_macros::{query, update};
use ic_cdk_timers::{clear_timer, set_timer_interval, TimerId};
use ic_utils::logger::log_message;
use ic_web3::types::H160;

use crate::{
    utils::{check_balance, collect_fee, publish::publish, rec_eth_addr, validate_caller},
    CandidSub, Chain, PythiaError, Sub, STATE,
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
    STATE.with(|state| {
        let state = state.borrow();

        if state.subs.len() as u64 >= state.subs_limit_total {
            return Err(anyhow!("Subs total limit reached"));
        }

        let mut subs_counter = 0;
        for sub in state.subs.iter() {
            if sub.owner == hex::encode(pub_key.as_bytes()) {
                subs_counter += 1;
            }
        }

        if subs_counter >= state.subs_limit_wallet {
            return Err(anyhow!("Subs limit per waller reached"));
        }

        Ok(())
    })
}

#[query]
pub fn get_subs(pub_key: Option<String>) -> Result<Vec<CandidSub>, String> {
    _get_subs(pub_key).map_err(|e| format!("{e:?}"))
}

fn _get_subs(pub_key: Option<String>) -> Result<Vec<CandidSub>> {
    let subs: Vec<CandidSub> = STATE.with(|state| {
        state
            .borrow()
            .subs
            .iter()
            .filter(|s| {
                if let Some(pub_key) = pub_key.clone() {
                    s.owner == pub_key
                } else {
                    true
                }
            })
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
    let chain = get_chain(&chain_id)?;
    let pub_key = rec_eth_addr(&msg, &sig).await?;
    let exec_addr = get_exec_addr_from_pub(&pub_key).await?;

    check_balance(&exec_addr, &chain).await?;

    STATE.with(|state| {
        let mut state = state.borrow_mut();

        for sub in state.subs.iter_mut() {
            if sub.owner == hex::encode(pub_key.as_bytes()) {
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

pub fn get_chain(chain_id: &Nat) -> Result<Chain> {
    STATE.with(|state| {
        Ok(state
            .borrow()
            .chains
            .get(chain_id)
            .ok_or(PythiaError::ChainDoesNotExist)?
            .clone())
    })
}

pub fn add_sub(sub: &Sub) {
    STATE.with(|state| state.borrow_mut().subs.push(sub.clone()))
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

    STATE.with(|state| {
        let mut state = state.borrow_mut();

        let mut sub = state.subs
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

    STATE.with(|state| {
        let mut state = state.borrow_mut();

        let mut sub = state.subs
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

    STATE.with(|state| {
        let mut state = state.borrow_mut();
        for mut sub in state.subs.iter_mut() {
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

    STATE.with(|state| {
        let state = state.borrow();
        for sub in state.subs.iter() {
            let timer_id: TimerId =
                serde_json::from_str(&sub.timer_id).expect("should be valid timer id");

            clear_timer(timer_id);
        }
    });

    STATE.with(|s| s.borrow_mut().subs = vec![]);

    Ok(())
}
use std::{str::FromStr, time::Duration};

use anyhow::Result;

use ic_cdk::export::candid::Nat;
use ic_cdk_macros::{query, update};
use ic_cdk_timers::set_timer_interval;
use ic_web3::types::H160;
use ic_utils::logger::log_message;

use crate::{
    utils::{check_balance, collect_fee, publish::publish, rec_eth_addr},
    CandidSub, Chain, PythiaError, Sub, User, CHAINS, U256, USERS,
};

#[update]
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
    .map_err(|e| e.to_string())
}

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
    let chain_id = U256::from(chain_id);
    let chain = get_chain(&chain_id)?;
    let pub_key = rec_eth_addr(&msg, &sig).await?;
    let user = get_user(&pub_key)?;

    check_balance(&user, &chain).await?;
    collect_fee(&user, &chain).await?;

    let sub = Sub::instance(
        &chain,
        pair_id,
        &contract_addr,
        &method_abi,
        &frequency,
        &user,
        is_random,
    )
    .await?;
    add_sub(&sub, &pub_key);

    log_message(format!("[USER: {}] sub creation; sub id: {}", user.pub_key, sub.id));

    Ok(())
}

#[query]
pub fn get_subs(pub_key: String) -> Result<Vec<CandidSub>, String> {
    _get_subs(pub_key).map_err(|e| e.to_string())
}

fn _get_subs(pub_key: String) -> Result<Vec<CandidSub>> {
    let pub_key = H160::from_str(&pub_key)?;

    let subs = USERS.with(|users| {
        users
            .borrow()
            .get(&pub_key)
            .ok_or(PythiaError::UserNotFound)
            .map(|user| user.subs.clone())
    })?;

    Ok(subs.into_iter().map(|sub| sub.into()).collect())
}

#[update]
pub async fn refresh_subs(chain_id: Nat, msg: String, sig: String) -> Result<(), String> {
    _refresh_subs(chain_id, msg, sig)
        .await
        .map_err(|e| e.to_string())
}

async fn _refresh_subs(chain_id: Nat, msg: String, sig: String) -> Result<()> {
    let chain_id = U256::from(chain_id);
    let chain = get_chain(&chain_id)?;
    let pub_key = rec_eth_addr(&msg, &sig).await?;
    let user = get_user(&pub_key)?;

    check_balance(&user, &chain).await?;

    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&pub_key).expect("user should exist");

        for sub in user.subs.iter_mut() {
            let id = sub.id;

            let timer_id = set_timer_interval(Duration::from_secs(sub.frequency), move || {
                publish(id, pub_key);
            });

            sub.timer_id = serde_json::to_string(&timer_id).expect("should be valid timer id");

        }

        log_message(format!("[USER: {}] subs refresh; chain id: {}", pub_key, chain_id.0));

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

pub fn get_user(pub_key: &H160) -> Result<User> {
    USERS.with(|users| {
        Ok(users
            .borrow()
            .get(pub_key)
            .ok_or(PythiaError::UserNotFound)?
            .clone())
    })
}

pub fn add_sub(sub: &Sub, pub_key: &H160) {
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(pub_key).expect("user should exist");
        user.subs.push(sub.clone());
    });
}

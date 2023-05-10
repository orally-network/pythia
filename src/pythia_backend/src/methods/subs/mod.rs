use std::time::Duration;

use anyhow::Result;

use ic_cdk::export::candid::Nat;
use ic_cdk_macros::update;
use ic_cdk_timers::set_timer_interval;
use ic_web3::types::H160;

use crate::{
    utils::{check_balance, collect_fee, publish::publish, rec_eth_addr},
    Chain, PythiaError, Sub, User, CHAINS, U256, USERS,
};

#[update]
pub async fn subscribe(
    chain_id: Nat,
    contract_addr: String,
    method_abi: String,
    frequency: Nat,
    is_random: bool,
    msg: String,
    sig: String,
) -> Result<(), String> {
    _subscribe(
        chain_id,
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
        &contract_addr,
        &method_abi,
        &frequency,
        &user,
        is_random,
    )
    .await?;
    add_sub(&sub, &pub_key);

    Ok(())
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

            sub.timer_id = set_timer_interval(Duration::from_secs(sub.frequency), move || {
                publish(id, pub_key);
            });
        }

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

use std::time::Duration;

use anyhow::Result;

use ic_cdk::export::{candid::Nat, Principal};
use ic_cdk_macros::update;
use ic_cdk_timers::set_timer_interval;

use crate::{
    utils::{check_balance, collect_fee, publish::publish},
    Chain, PythiaError, Sub, User, CHAINS, U256, USERS,
};

#[update]
pub async fn subscribe(
    chain_id: Nat,
    contract_addr: String,
    method_abi: String,
    frequency: Nat,
    is_random: bool,
) -> Result<(), String> {
    let caller = ic_cdk::caller();
    let frequency = frequency
        .0
        .to_string()
        .parse::<u64>()
        .expect("valid number");

    let chain_id = U256::from(chain_id);
    let chain = get_chain(&chain_id).map_err(|e| format!("{}", e))?;

    let user = get_user(&caller).map_err(|e| format!("{}", e))?;

    check_balance(&user, &chain)
        .await
        .map_err(|e| format!("{}", e))?;

    collect_fee(&user, &chain)
        .await
        .map_err(|e| format!("{}", e))?;

    let sub = Sub::instance(
        &chain,
        &contract_addr,
        &method_abi,
        &frequency,
        &user,
        &caller,
        is_random,
    )
    .await
    .map_err(|e| format!("{}", e))?;

    add_sub(&sub, &caller);

    Ok(())
}

#[update]
pub async fn refresh_subs(chain_id: Nat) -> Result<(), String> {
    let caller = ic_cdk::caller();
    let chain_id = U256::from(chain_id);
    let chain = get_chain(&chain_id).map_err(|e| format!("{}", e))?;

    let user = get_user(&caller).map_err(|e| format!("{}", e))?;

    check_balance(&user, &chain)
        .await
        .map_err(|e| format!("{}", e))?;

    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&caller).expect("user should exist");

        for sub in user.subs.iter_mut() {
            let id = sub.id;

            sub.timer_id = set_timer_interval(Duration::from_secs(sub.frequency), move || {
                publish(id, caller);
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

pub fn get_user(caller: &Principal) -> Result<User> {
    USERS.with(|users| {
        Ok(users
            .borrow()
            .get(caller)
            .ok_or(PythiaError::UserNotFound)?
            .clone())
    })
}

pub fn add_sub(sub: &Sub, caller: &Principal) {
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(caller).expect("user should exist");
        user.subs.push(sub.clone());
    });
}

use std::{time::Duration, vec};

use anyhow::{Context, Result};
use ic_cdk::export::candid::Nat;
use ic_cdk_timers::set_timer;
use ic_dl_utils::retry_until_success;

use futures::future::join_all;

use super::withdraw;
use crate::{
    clone_with_state, log,
    types::{
        balance::Balances,
        errors::PythiaError,
        subscription::{Subscription, Subscriptions}, timer::Timer,
    },
    utils::{
        abi, address, canister,
        multicall::{multicall, Call},
        nat, web3,
    },
};

pub fn execute() {
    ic_cdk::spawn(async {
        if let Err(e) = _execute().await {
            log!("error while executing publisher job: {e:?}");
        }
    })
}

async fn _execute() -> Result<()> {
    log!("[PUBLISHER] publisher job started");
    Timer::activate()
        .context(PythiaError::UnableToActivateTimer)?;

    Subscriptions::stop_insufficients()
        .context(PythiaError::UnableToStopInsufficientSubscriptions)?;

    let mut futures = vec![];
    let (publishable_subs, is_active) = Subscriptions::get_publishable();
    publishable_subs
        .iter()
        .for_each(|(chain_id, subs)| {
            if subs.is_empty() {
                return;
            }

            futures.push(publish_on_chain(chain_id.clone(), subs.clone()));
        });

    if !is_active {
        withdraw::withdraw().await;
        Timer::deactivate()
            .context(PythiaError::UnableToDeactivateTimer)?;
        log!("[PUBLISHER] publisher job stopped");
        return Ok(());
    }

    join_all(futures).await.iter().for_each(|e| {
        if let Err(e) = e {
            log!("[PUBLISHER] error while publishing: {e:?}");
        }
    });

    withdraw::withdraw().await;

    let timer_id = set_timer(
        Duration::from_secs(nat::to_u64(&clone_with_state!(timer_frequency))),
        execute,
    );

    Timer::update(timer_id)
        .context(PythiaError::UnableToUpdateTimer)?;

    log!("[PUBLISHER] publisher job executed");
    Ok(())
}

async fn publish_on_chain(chain_id: Nat, mut subscriptions: Vec<Subscription>) -> Result<()> {
    log!("[PUBLISHER] chain: {}, publishing", chain_id);
    let w3 = web3::instance(&chain_id)?;
    let pma = canister::pma().await.context(PythiaError::UnableToGetPMA)?;
    while !subscriptions.is_empty() {
        log!("[PUBLISHER] chain: {}, subscriptions left: {}", chain_id, subscriptions.len());
        let calls: Vec<Call> = join_all(subscriptions.iter().map(|sub| async {
            Call {
                target: address::to_h160(&sub.contract_addr).expect("should be valid address"),
                call_data: abi::get_call_data(&sub.method).await,
                gas_limit: nat::to_u256(&sub.method.gas_limit),
            }
        }))
        .await;

        let fee = canister::fee(&chain_id).await?;
        let mut gas_price = retry_until_success!(w3.eth().gas_price())?;
        gas_price = (gas_price / 10) * 12;
        let multicall_results = multicall(&w3, &chain_id, calls.clone(), gas_price)
            .await
            .context(PythiaError::UnableToExecuteMulticall)?;

        if multicall_results.is_empty() {
            log!("[PUBLISHER] chain: {}, no results from multicall, corruption detected", chain_id);
            continue;
        }

        subscriptions = multicall_results
            .iter()
            .zip(subscriptions)
            .filter(|(result, sub)| {
                if nat::from_u256(&result.used_gas) > sub.method.gas_limit {
                    log!("[PUBLISHER] chain: {}, gas limit exceeded for sub {}", chain_id, sub.id);
                    Subscriptions::stop(&chain_id, &sub.owner, &sub.id).expect("should stop sub");
                    return false;
                }

                if result.used_gas == 0.into() {
                    return true;
                }

                Subscriptions::update_last_update(&chain_id, &sub.id);
                let mut amount = nat::from_u256(&gas_price) * (sub.method.gas_limit.clone() + 100);
                amount += fee.clone();
                Balances::reduce(&chain_id, &sub.owner, &amount)
                    .expect("should reduce balance");
                canister::collect_fee(&chain_id, &pma, &fee)
                    .expect("should collect fee");

                false
            })
            .map(|(_, subscription)| subscription)
            .collect();
    }
    log!("[PUBLISHER] chain: {}, published", chain_id);
    Ok(())
}

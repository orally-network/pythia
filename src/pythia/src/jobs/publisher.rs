use std::time::Duration;

use anyhow::{Context, Result};
use ic_cdk::export::candid::Nat;
use ic_cdk_timers::set_timer;

use futures::future::join_all;

use super::{subscriptions_grouper, withdraw};
use crate::{
    clone_with_state, log, retry_until_success,
    types::{
        balance::Balances,
        errors::PythiaError,
        logger::PUBLISHER,
        subscription::{Subscription, Subscriptions, UpdateSubscriptionRequest},
        timer::Timer,
    },
    utils::{
        abi, address, canister,
        multicall::{multicall, Call},
        nat, web3, time,
    },
};

pub fn execute() {
    ic_cdk::spawn(async {
        if let Err(e) = _execute().await {
            log!("[{PUBLISHER}] error while executing publisher job: {e:?}");
        }
    })
}

async fn _execute() -> Result<()> {
    log!("[{PUBLISHER}] publisher job started");
    Timer::activate().context(PythiaError::UnableToActivateTimer)?;

    subscriptions_grouper::group()?;

    let (publishable_subs, is_active) = Subscriptions::get_publishable();    

    let futures = publishable_subs
        .into_iter()
        .filter(|(_, subs)| !subs.is_empty())
        .map(|(chain_id, subs)| publish_on_chain(chain_id, subs))
        .collect::<Vec<_>>();

    let should_stop_insufficient_subs = !futures.is_empty();

    if !is_active {
        withdraw::withdraw().await;
        Timer::deactivate().context(PythiaError::UnableToDeactivateTimer)?;
        log!("[{PUBLISHER}] publisher job stopped");
        return Ok(());
    }

    join_all(futures).await.iter().for_each(|e| {
        if let Err(e) = e {
            log!("[{PUBLISHER}] error while publishing: {e:?}");
        }
    });

    if should_stop_insufficient_subs {
        Subscriptions::stop_insufficients()
            .await
            .context(PythiaError::UnableToStopInsufficientSubscriptions)?;
    }

    withdraw::withdraw().await;

    let timer_id = set_timer(
        Duration::from_secs(nat::to_u64(&clone_with_state!(timer_frequency))),
        execute,
    );

    Timer::update(timer_id).context(PythiaError::UnableToUpdateTimer)?;

    log!("[{PUBLISHER}] publisher job executed");
    Ok(())
}

async fn publish_on_chain(chain_id: Nat, mut subscriptions: Vec<Subscription>) -> Result<()> {
    let publishing_time = time::in_seconds();
    log!("[{PUBLISHER}] chain: {}, publishing", chain_id);
    let w3 = web3::instance(&chain_id)?;
    let pma = canister::pma().await.context(PythiaError::UnableToGetPMA)?;
    while !subscriptions.is_empty() {
        log!(
            "[{PUBLISHER}] chain: {}, subscriptions left: {}",
            chain_id,
            subscriptions.len()
        );

        let mut calls = vec![];
        for sub in subscriptions.iter() {
            let call = Call {
                target: address::to_h160(&sub.contract_addr)
                    .context(PythiaError::InvalidAddressFormat)?,
                call_data: abi::get_call_data(&sub.method)
                    .await
                    .context(PythiaError::UnableToFormCallData)?,
                gas_limit: nat::to_u256(&sub.method.gas_limit),
            };

            calls.push(call);
        }

        let fee = canister::fee(&chain_id).await.context("Unable to get fe")?;
        let mut gas_price = retry_until_success!(w3.eth().gas_price(canister::transform_ctx()))?;
        // multiply the gas_price to 1.2 to avoid long transaction confirmation
        gas_price = (gas_price / 10) * 12;
        let multicall_results = multicall(&w3, &chain_id, calls.clone(), gas_price)
            .await
            .context(PythiaError::UnableToExecuteMulticall)?;

        if multicall_results.is_empty() {
            log!(
                "[{PUBLISHER}] chain: {}, no results from multicall, corruption detected",
                chain_id
            );
            continue;
        }

        subscriptions = multicall_results
            .iter()
            .zip(subscriptions)
            .filter_map(|(result, sub)| {
                let mut used_gas = nat::from_u256(&result.used_gas);
                #[allow(clippy::cmp_owned)]
                if used_gas == Nat::from(0) {
                    return Some(sub);
                }

                if used_gas > sub.method.gas_limit {
                    log!(
                        "[{PUBLISHER}] chain: {}, gas limit exceeded for sub {}",
                        chain_id,
                        sub.id
                    );
                    Subscriptions::stop(&chain_id, &sub.owner, &sub.id).expect("should stop sub");
                    // inscrease gas limit by 30 persent
                    let new_gas_limit = (used_gas.clone() / 10) / 13;
                    Subscriptions::update(
                        &UpdateSubscriptionRequest {
                            chain_id: chain_id.clone(),
                            id: sub.id.clone(),
                            gas_limit: Some(new_gas_limit),
                            ..Default::default()
                        },
                        &sub.owner,
                    )
                    .expect("should update sub");
                }

                Subscriptions::update_last_update(&chain_id, &sub.id, !result.success, publishing_time);
                let gas_for_tx = (web3::TRANSFER_GAS_LIMIT / multicall_results.len() as u64) + 100;
                used_gas += gas_for_tx;

                let mut amount = nat::from_u256(&gas_price) * (used_gas);
                amount += fee.clone();

                Balances::reduce(&chain_id, &sub.owner, &amount).expect("should reduce balance");
                canister::collect_fee(&chain_id, &pma, &fee).expect("should collect fee");

                None
            })
            .collect();
    }

    log!("[{PUBLISHER}] chain: {}, published", chain_id);
    Ok(())
}

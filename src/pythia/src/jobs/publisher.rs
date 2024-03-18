use std::time::Duration;

use anyhow::{Context, Result};
use candid::Nat;
use ic_cdk_timers::set_timer;

use futures::future::join_all;
use thiserror::Error;

use super::{subscriptions_grouper, withdraw};
use crate::{
    clone_with_state, log, retry_until_success,
    types::{
        balance::Balances,
        chains::Chains,
        errors::PythiaError,
        logger::PUBLISHER,
        subscription::{Subscription, Subscriptions, UpdateSubscriptionRequest},
        timer::Timer,
    },
    utils::{
        abi, address, canister,
        multicall::{multicall, Call},
        nat, time, web3,
    },
};

#[derive(Error, Debug)]
enum PublishOnChainError {
    #[error("Chain error")]
    ChainError(anyhow::Error),
    #[error("Subscription error")]
    SubscriptionError { err: anyhow::Error, sub_id: Nat },
}

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

    let timer_id = set_timer(
        Duration::from_secs(nat::to_u64(&clone_with_state!(timer_frequency))),
        execute,
    );

    Timer::update(timer_id).context(PythiaError::UnableToUpdateTimer)?;

    subscriptions_grouper::group()?;

    let (publishable_subs, is_active) = Subscriptions::get_publishable().await;

    log!(
        "[{PUBLISHER}] Got publishable subs: len: {:#?}, is_active: {}",
        publishable_subs.len(),
        is_active
    );

    let futures = publishable_subs
        .clone()
        .into_iter()
        .filter(|(_, subs)| !subs.is_empty())
        .map(|(chain_id, subs)| publish_on_chain(chain_id, subs))
        .collect::<Vec<_>>();

    log!("[{PUBLISHER}] Futures created: len = {:#?}", futures.len());

    let should_stop_insufficient_subs = !futures.is_empty();

    if !is_active {
        withdraw::withdraw().await;
        Timer::deactivate().context(PythiaError::UnableToDeactivateTimer)?;
        log!("[{PUBLISHER}] Subscription is inactive, publisher job stopped");
        return Ok(());
    }

    join_all(futures)
        .await
        .iter()
        .zip(publishable_subs)
        .for_each(|(e, sub)| {
            if let Err(e) = e {
                log!("[{PUBLISHER}] error while publishing: {e:?}");
                match e {
                    PublishOnChainError::ChainError(_) => {
                        Chains::increment_error_count(sub.0).expect("Shouldn't fail");
                    }
                    PublishOnChainError::SubscriptionError { sub_id, .. } => {
                        Subscriptions::update_last_update(
                            &sub.0,
                            &sub_id,
                            true,
                            time::in_seconds(),
                        );
                    }
                }
            } else {
                Chains::reset_error_count(sub.0).expect("Shouldn't fail");
            }
        });

    if should_stop_insufficient_subs {
        match Subscriptions::stop_insufficients()
            .await
            .context(PythiaError::UnableToStopInsufficientSubscriptions)
        {
            Ok(_) => log!("[{PUBLISHER}] stopped insufficient subscriptions"),
            Err(e) => log!("[{PUBLISHER}] error while stopping insufficient subscriptions: {e:?}"),
        }
    }

    withdraw::withdraw().await;

    log!("[{PUBLISHER}] publisher job executed");
    Ok(())
}

async fn publish_on_chain(
    chain_id: Nat,
    mut subscriptions: Vec<Subscription>,
) -> Result<(), PublishOnChainError> {
    let publishing_time = time::in_seconds();
    log!("[{PUBLISHER}] chain: {}, publishing", chain_id);

    let w3 = web3::instance(&chain_id).map_err(PublishOnChainError::ChainError)?;
    let pma = canister::pma()
        .await
        .context(PythiaError::UnableToGetPMA)
        .map_err(PublishOnChainError::ChainError)?;

    while !subscriptions.is_empty() {
        log!(
            "[{PUBLISHER}] chain: {}, subscriptions left: {}",
            chain_id,
            subscriptions.len()
        );

        let calls = get_calls_from_subs(&subscriptions).await?;

        log!("[{PUBLISHER}] Calls inited, chain: {}", chain_id);

        let fee = canister::fee(&chain_id) // TODO: move out of loop
            .await
            .context("Unable to get fee")
            .map_err(PublishOnChainError::ChainError)?;

        log!("[{PUBLISHER}] Trying to get gas_price: {}", chain_id);

        let mut gas_price =
            match retry_until_success!(w3.eth().gas_price(canister::transform_ctx())) {
                Ok(gas_price) => gas_price,
                Err(e) => {
                    log!("Unable to get gas_price: {e:?}");
                    Err(e)
                        .context(PythiaError::UnableToGetGasPrice)
                        .map_err(PublishOnChainError::ChainError)?
                }
            };
        log!(
            "[{PUBLISHER}] chain: {}, gas price: {}, fee: {}",
            chain_id,
            gas_price,
            fee
        );

        // multiply the gas_price to 1.2 to avoid long transaction confirmation
        gas_price = (gas_price / 10) * 12;
        let multicall_results = multicall(&w3, &chain_id, calls.clone(), gas_price)
            .await
            .context(PythiaError::UnableToExecuteMulticall)
            .map_err(PublishOnChainError::ChainError)?;

        if multicall_results.is_empty() {
            log!(
                "[{PUBLISHER}] chain: {}, no results from multicall, corruption detected",
                chain_id
            );
            continue;
        }

        let mut remaining_subs = vec![];

        for (result, sub) in multicall_results.iter().zip(subscriptions) {
            let mut used_gas = nat::from_u256(&result.used_gas);

            log!(
                "[{PUBLISHER}] chain: {}, sub: {}, used gas: {}, gas limit: {}",
                chain_id,
                sub.id,
                used_gas,
                sub.method.gas_limit
            );

            #[allow(clippy::cmp_owned)]
            if used_gas == Nat::from(0) {
                remaining_subs.push(sub);
                continue;
            }

            if used_gas > sub.method.gas_limit {
                log!(
                    "[{PUBLISHER}] chain: {}, gas limit exceeded for sub {}",
                    chain_id,
                    sub.id
                );
                Subscriptions::stop(&chain_id, &sub.owner, &sub.id).expect("should stop sub");
                // inscrease gas limit by 30 persent
                let new_gas_limit = (used_gas.clone() / 10) * 13;
                Subscriptions::update(
                    &UpdateSubscriptionRequest {
                        chain_id: chain_id.clone(),
                        id: sub.id.clone(),
                        gas_limit: Some(new_gas_limit),
                        ..Default::default()
                    },
                    &sub.owner,
                )
                .await
                .expect("should update sub");
            }

            Subscriptions::update_last_update(&chain_id, &sub.id, !result.success, publishing_time);
            let gas_for_tx = (web3::TRANSFER_GAS_LIMIT / multicall_results.len() as u64) + 100;
            used_gas += gas_for_tx;

            let mut amount = nat::from_u256(&gas_price) * (used_gas);
            amount += fee.clone();

            Balances::reduce(&chain_id, &sub.owner, &amount).expect("should reduce balance");
            canister::collect_fee(&chain_id, &pma, &fee).expect("should collect fee");
        }

        subscriptions = remaining_subs;
    }

    log!("[{PUBLISHER}] chain: {}, published", chain_id);
    Ok(())
}

async fn get_calls_from_subs(subs: &[Subscription]) -> Result<Vec<Call>, PublishOnChainError> {
    let mut calls = Vec::with_capacity(subs.len());

    for sub in subs {
        let call = Call {
            target: address::to_h160(&sub.contract_addr)
                .context(PythiaError::InvalidAddressFormat)
                .map_err(|err| PublishOnChainError::SubscriptionError {
                    err,
                    sub_id: sub.id.clone(),
                })?,
            call_data: abi::get_call_data(&sub.method)
                .await
                .context(PythiaError::UnableToFormCallData)
                .map_err(|err| PublishOnChainError::SubscriptionError {
                    err,
                    sub_id: sub.id.clone(),
                })?,
            gas_limit: nat::to_u256(&sub.method.gas_limit),
        };

        calls.push(call);
    }

    Ok(calls)
}

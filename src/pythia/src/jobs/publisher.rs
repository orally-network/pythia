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
        subscription::{Subscription, Subscriptions},
    },
    update_state,
    utils::{
        abi, address,
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
    log!("publisher job started");
    update_state!(is_timer_active, true);

    Subscriptions::stop_insufficients()
        .context(PythiaError::UnableToStopInsufficientSubscriptions)?;

    let mut futures = vec![];
    Subscriptions::get_publishable()
        .iter()
        .for_each(|(chain_id, subs)| {
            if subs.is_empty() {
                return;
            }

            futures.push(publish_on_chain(chain_id.clone(), subs.clone()));
        });

    if futures.is_empty() {
        withdraw::withdraw().await;
        update_state!(is_timer_active, false);
        log!("publisher job stopped");
        return Ok(());
    }

    join_all(futures).await.iter().for_each(|e| {
        if let Err(e) = e {
            log!("error while publishing: {e:?}");
        }
    });

    withdraw::withdraw().await;

    set_timer(
        Duration::from_secs(nat::to_u64(&clone_with_state!(timer_frequency))),
        execute,
    );

    log!("publisher job executed");
    Ok(())
}

async fn publish_on_chain(chain_id: Nat, mut subscriptions: Vec<Subscription>) -> Result<()> {
    let w3 = web3::instance(&chain_id)?;
    while !subscriptions.is_empty() {
        let calls: Vec<Call> = join_all(subscriptions.iter().map(|sub| async {
            Call {
                target: address::to_h160(&sub.contract_addr).expect("should be valid address"),
                call_data: abi::get_call_data(&sub.method).await,
                gas_limit: nat::to_u256(&sub.method.gas_limit),
            }
        }))
        .await;

        let gas_price = retry_until_success!(w3.eth().gas_price())?;
        subscriptions = multicall(&w3, &chain_id, calls, gas_price)
            .await
            .context(PythiaError::UnableToExecuteMulticall)?
            .iter()
            .zip(subscriptions)
            .filter(|(result, sub)| {
                if nat::from_u256(&result.used_gas) > sub.method.gas_limit {
                    log!("gas limit exceeded for sub {}", sub.id);
                    Subscriptions::stop(&chain_id, &sub.owner, &sub.id)
                        .expect("should stop sub");
                    return false;
                }

                if result.used_gas != 0.into() {
                    return true;
                }

                Subscriptions::update_last_update(&chain_id, &sub.id);
                let amount = nat::from_u256(&gas_price) * (sub.method.gas_limit.clone()+100);
                Balances::reduce(&chain_id, &sub.owner, &amount)
                    .expect("should reduce balance");

                false
            })
            .map(|(_, subscription)| subscription)
            .collect();
    }

    Ok(())
}

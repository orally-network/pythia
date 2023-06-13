use std::{str::FromStr, vec, time::Duration};

use anyhow::{Result, anyhow};
use ic_dl_utils::evm::time_in_seconds;
use ic_utils::logger::log_message;
use ic_cdk_timers::set_timer;
use ic_cdk::{export::candid::Nat, api::management_canister::main::raw_rand};

use futures::future::join_all;
use ic_web3::{types::H160, ethabi::{Function, Token}};

use super::withdraw;
use crate::{update_state, types::{subscription::Subscription, methods::{Method, MethodType}}, clone_with_state, utils::{multicall::{Call, multicall}, nat_to_u256, get_web3, u256_to_nat, cast_to_param_type, sybil::get_asset_data}, STATE};

const BITS_IN_BYTE: usize = 8;

pub fn execute() {
    ic_cdk::spawn(_execute())
}

async fn _execute() {
    ic_cdk::println!("publisher job started");
    log_message("publisher job started".into());

    update_state!(is_timer_active, true);
    let mut futures = vec![];

    for (chain_id, subs) in clone_with_state!(subscriptions) {
        let active_subs = subs
            .into_iter()
            .filter(|sub| {
                sub.status.is_active
            })
            .collect::<Vec<Subscription>>();
        if !active_subs.is_empty() {
            log_message(format!("publishing on chain {}", chain_id));
            let expired_subs = active_subs
                .iter()
                .filter(|sub| {
                    sub.status.last_update.clone() + sub.frequency.clone() <= Nat::from(time_in_seconds())
                })
                .cloned()
                .collect();

            futures.push(publish_on_chain(chain_id, expired_subs));
        }
    }

    if futures.is_empty() {
        update_state!(is_timer_active, false);
        return 
    }

    join_all(futures)
        .await
        .iter()
        .for_each(|e| {
            if let Err(e) = e {
                ic_cdk::println!("error while publishing: {e:?}");
                log_message(format!("error while publishing: {e:?}"));
            }
        });

    withdraw::withdraw().await;
    
    set_timer(Duration::from_secs(clone_with_state!(timer_frequency)), execute);

    ic_cdk::println!("publisher job executed");
    log_message("publisher job executed".into());
}

async fn publish_on_chain(chain_id: Nat, mut subscriptions: Vec<Subscription>) -> Result<()> {
    let w3 = get_web3(&chain_id)?;
    while !subscriptions.is_empty() {
        let calls: Vec<Call> = join_all(subscriptions
            .iter()
            .map(|sub| async {
                Call {
                    target: H160::from_str(&sub.contract_addr).expect("should be valid address"),
                    call_data: get_call_data(&sub.method).await,
                    gas_limit: nat_to_u256(&sub.method.gas_limit),
                }
            })
        ).await;
        
        let results = multicall(&w3, &chain_id, calls)
            .await?;

        let remove_indexes: Vec<Nat> = results
            .iter()
            .zip(subscriptions.iter())
            .filter(|(result, sub)| {
                if u256_to_nat(result.used_gas) > sub.method.gas_limit {
                    ic_cdk::println!("gas limit exceeded for sub {}", sub.id);
                    log_message(format!("gas limit exceeded for sub {}", sub.id));
                    stop_subscription(&sub.id, &chain_id);
                    return true;
                }

                if result.used_gas != 0.into() {
                    return true;
                }

                false
            })
            .map(|(_, sub)| sub.id.clone())
            .collect();
        
        subscriptions = subscriptions.iter()
            .filter(|sub| {
                if remove_indexes.contains(&sub.id) {
                    update_last_update(&chain_id, &sub.id);
                    return false;
                }

                true
            })
            .map(|sub| sub.clone())
            .collect();
    }

    Ok(())
}

fn update_last_update(chain_id: &Nat, sub_id: &Nat) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .subscriptions
            .get_mut(chain_id)
            .expect("chain should exist")
            .iter_mut()
            .find(|sub| sub.id == *sub_id)
            .expect("sub should exist")
            .status
            .last_update = time_in_seconds().into();
    });
}

async fn get_call_data(method: &Method) -> Vec<u8> {
    let input = get_input(&method.method_type)
        .await
        .expect("should be valid input");

    serde_json::from_str::<Function>(&method.abi)
        .expect("should be valid abi")
        .encode_input(&input)
        .expect("should encode input")
}

pub async fn get_input(method_type: &MethodType) -> Result<Vec<Token>> {
    let input = match method_type {
        MethodType::Pair(pair_id) => get_sybil_input(pair_id).await?,
        MethodType::Random(abi_type) => vec![get_random_input(abi_type).await?],
        MethodType::Empty => vec![],
    };

    Ok(input)
}

async fn get_random_input(abi_type: &str) -> Result<Token> {
    let (mut raw_data,) = raw_rand().await.expect("random should be generated");

    let (insufficient_bytes_count, was_overflowed) = raw_data.len().overflowing_sub(BITS_IN_BYTE);

    if was_overflowed {
        raw_data.append(&mut vec![0; insufficient_bytes_count]);
    }

    let value = u64::from_be_bytes(
        raw_data[..BITS_IN_BYTE]
            .try_into()
            .expect("should be valid convertation"),
    );

    cast_to_param_type(value, abi_type).ok_or(anyhow!("invalid abi type"))
}

async fn get_sybil_input(pair_id: &str) -> Result<Vec<Token>> {
    let rate = get_asset_data(pair_id).await?;

    Ok(vec![
        Token::String(rate.symbol),
        Token::Uint(rate.rate.into()),
        Token::Uint(rate.decimals.into()),
        Token::Uint(rate.timestamp.into()),
    ])
}

fn stop_subscription(sub_id: &Nat, chain_id: &Nat) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .subscriptions
            .get_mut(chain_id)
            .expect("chain should exist")
            .iter_mut()
            .find(|sub| sub.id == *sub_id)
            .expect("sub should exist")
            .status
            .is_active = false;
    })
}

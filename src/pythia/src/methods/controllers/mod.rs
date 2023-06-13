use anyhow::Result;

use ic_cdk::{
    api::management_canister::main::{canister_status, CanisterIdRecord},
    export::{candid::Nat, Principal},
};
use ic_cdk_macros::update;
use ic_utils::logger::log_message;

use crate::{
    utils::{validate_caller, nat_to_u64}, STATE, jobs::{withdraw, publisher},
};

#[update]
fn update_tx_fee(tx_fee: Nat) -> Result<(), String> {
    validate_caller().map_err(|e| format!("{}", e))?;
    STATE.with(|state| state.borrow_mut().tx_fee = tx_fee.clone());
    log_message(format!("[TX FEE] updating: {}", tx_fee));
    Ok(())
}

#[update]
pub fn update_subs_limit_wallet(limit: Nat) -> Result<(), String> {
    _update_subs_limit_wallet(limit).map_err(|e| format!("{e:?}"))
}

fn _update_subs_limit_wallet(limit: Nat) -> Result<()> {
    validate_caller()?;
    STATE.with(|s| s.borrow_mut().subs_limit_wallet = nat_to_u64(&limit));
    log_message(format!("[SUBS LIMIT WALLET] updating: {}", limit));
    Ok(())
}

#[update]
pub fn update_subs_limit_total(limit: Nat) -> Result<(), String> {
    _update_subs_limit_total(limit).map_err(|e| format!("{e:?}"))
}

fn _update_subs_limit_total(limit: Nat) -> Result<()> {
    validate_caller()?;
    STATE.with(|s| s.borrow_mut().subs_limit_total = nat_to_u64(&limit));
    log_message(format!("[SUBS LIMIT TOTAL] updating: {}", limit));
    Ok(())
}

#[update]
pub async fn update_controllers() -> Vec<Principal> {
    let canister_id_record = CanisterIdRecord {
        canister_id: ic_cdk::id(),
    };

    let (canister_status,) = canister_status(canister_id_record)
        .await
        .expect("should execute in the IC environment");

    STATE.with(|state| {
        state.borrow_mut().controllers = canister_status.settings.controllers.clone();
    });

    canister_status.settings.controllers
}

#[update]
pub fn update_timer_frequency(frequency: Nat) -> Result<(), String> {
    _update_timer_frequency(frequency).map_err(|e| format!("{e:?}"))
}

fn _update_timer_frequency(frequency: Nat) -> Result<()> {
    validate_caller()?;
    STATE.with(|state| state.borrow_mut().timer_frequency = nat_to_u64(&frequency));
    log_message(format!("[TIMER FREQUENCY] updating: {}", frequency));
    Ok(())
}

#[update]
pub fn execute_withdraw() -> Result<(), String> {
    _execute_withdraw().map_err(|e| format!("{e:?}"))
}

fn _execute_withdraw() -> Result<()> {
    validate_caller()?;
    withdraw::execute();
    Ok(())
}

#[update]
pub fn execute_publisher() -> Result<(), String> {
    _execute_publisher().map_err(|e| format!("{e:?}"))
}

fn _execute_publisher() -> Result<()> {
    validate_caller()?;
    publisher::execute();
    Ok(())
}
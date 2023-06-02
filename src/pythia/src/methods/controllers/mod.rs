use anyhow::{Context, Result};

use ic_cdk::{
    api::management_canister::main::{canister_status, CanisterIdRecord},
    export::{candid::Nat, Principal},
};
use ic_cdk_macros::update;
use ic_utils::logger::log_message;

use crate::{
    utils::validate_caller, STATE,
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
    let limit = *limit
        .0
        .to_u64_digits()
        .last()
        .context("limit should be greater than 1")?;

    STATE.with(|s| s.borrow_mut().subs_limit_wallet = limit);
    Ok(())
}

#[update]
pub fn update_subs_limit_total(limit: Nat) -> Result<(), String> {
    _update_subs_limit_total(limit).map_err(|e| format!("{e:?}"))
}

fn _update_subs_limit_total(limit: Nat) -> Result<()> {
    validate_caller()?;
    let limit = *limit
        .0
        .to_u64_digits()
        .last()
        .context("limit should be greater than 1")?;

    STATE.with(|s| s.borrow_mut().subs_limit_total = limit);
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

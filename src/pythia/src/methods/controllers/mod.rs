use anyhow::{Context, Result};

use ic_cdk::{
    api::management_canister::main::{canister_status, CanisterIdRecord},
    export::{candid::Nat, Principal},
};
use ic_cdk_macros::update;
use ic_utils::logger::log_message;

use crate::{
    utils::validate_caller, CONTROLLERS, SUBS_LIMIT_TOTAL, SUBS_LIMIT_WALLET, TX_FEE, U256,
};

#[update]
fn update_tx_fee(tx_fee: Nat) -> Result<(), String> {
    validate_caller().map_err(|e| format!("{}", e))?;

    TX_FEE.with(|tx_fee_state| {
        let tx_fee = U256::from(tx_fee);

        *tx_fee_state.borrow_mut() = tx_fee;

        log_message(format!("[TX FEE] updating: {}", tx_fee.0));
    });

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

    SUBS_LIMIT_WALLET.with(|s| *s.borrow_mut() = limit);
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

    SUBS_LIMIT_TOTAL.with(|s| *s.borrow_mut() = limit);
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

    CONTROLLERS.with(|controllers| {
        *controllers.borrow_mut() = canister_status.settings.controllers;
    });

    CONTROLLERS.with(|controllers| controllers.borrow().clone())
}

use anyhow::Result;

use ic_cdk::{
    api::management_canister::main::{canister_status, CanisterIdRecord},
    export::{candid::Nat, Principal},
};
use ic_cdk_macros::update;

use crate::{utils::validate_caller, CONTROLLERS, TX_FEE, U256};

#[update]
fn update_tx_fee(tx_fee: Nat) -> Result<(), String> {
    validate_caller().map_err(|e| format!("{}", e))?;

    TX_FEE.with(|tx_fee_state| {
        *tx_fee_state.borrow_mut() = U256::from(tx_fee);
    });

    Ok(())
}

#[update]
pub async fn get_controllers() -> Vec<Principal> {
    let controllers = CONTROLLERS.with(|controllers| controllers.borrow().clone());

    if !controllers.is_empty() {
        return controllers;
    }

    let canister_id_record = CanisterIdRecord {
        canister_id: ic_cdk::id(),
    };

    let (canister_status,) = canister_status(canister_id_record)
        .await
        .expect("should get canister status");

    CONTROLLERS.with(|controllers| {
        *controllers.borrow_mut() = canister_status.settings.controllers;
    });

    CONTROLLERS.with(|controllers| controllers.borrow().clone())
}

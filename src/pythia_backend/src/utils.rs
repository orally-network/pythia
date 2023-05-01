use anyhow::{Result, anyhow};

use ic_cdk_macros::update;
use ic_cdk::export::Principal;
use ic_cdk::api::management_canister::main::{
    canister_status,
    CanisterIdRecord
};

use crate::{
    CONTROLLERS, 
    types::errors::PythiaError
};

pub fn validate_caller() -> Result<()> {
    let controllers = CONTROLLERS.with(|controllers| {
        controllers.borrow().clone()
    });

    if controllers.contains(&ic_cdk::caller()) {
        return Ok(());
    }

    Err(anyhow!(PythiaError::NotAController))
}

#[update]
pub async fn get_controllers() -> Vec<Principal> {
    let controllers = CONTROLLERS.with(|controllers| {
        controllers.borrow().clone()
    });

    if !controllers.is_empty() {
        return controllers;
    }

    let canister_id_record = CanisterIdRecord{
        canister_id: ic_cdk::id(),
    };

    let (canister_status, ) = canister_status(canister_id_record)
        .await
        .expect("Failed to get canister status");

    CONTROLLERS.with(|controllers| {
        *controllers.borrow_mut() = canister_status.settings.controllers;
    });

    CONTROLLERS.with(|controllers| {
        controllers.borrow().clone()
    })
}
use std::str::FromStr;

use url::Url;
use time::OffsetDateTime;
use anyhow::{Result, Context, anyhow};
use siwe::{Message, VerificationOpts};

use ic_cdk_macros::update;
use ic_cdk::{
    export::Principal,
    api::management_canister::main::{
        canister_status,
        CanisterIdRecord
    },
};
use ic_web3::{
    Web3,
    types::H160,
    transports::ICHttp,
};

use crate::{
    U256,
    CONTROLLERS,
    MIN_BALANCE,
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

pub async fn rec_eth_addr(msg: &str, sig: &str) -> Result<H160> {
    let msg = Message::from_str(msg)
        .context("failed to parse the message")?;

    let sig = hex::decode(sig)
        .context("failed to decode the signature")?;
    
    let timestamp = OffsetDateTime::from_unix_timestamp((ic_cdk::api::time() / 1_000_000_000) as i64)
        .context("failed to get the timestamp")?;

    let opts = VerificationOpts {
        timestamp: Some(timestamp),
        ..Default::default()
    };

    msg.verify(&sig, &opts)
        .await
        .context("failed to verify the signature")?;

    Ok(H160::from_slice(&msg.address))
}

pub async fn check_balance(address: &H160, rpc: &Url) -> Result<()> {
    let balance = get_balance(address, rpc).await?;

    MIN_BALANCE.with(|min_balance| {
        if balance < *min_balance.borrow() {
            return Err(anyhow!(PythiaError::InsufficientBalance));
        }

        Ok(())
    })
}

pub async fn get_balance(address: &H160, rpc: &Url) -> Result<U256> {
    let w3 = Web3::new(
        ICHttp::new(rpc.as_str(), None, None)
            .context("failed to connect to a node")?
    );

    let balance = w3.eth()
        .balance(address.clone(), None)
        .await
        .context("failed to get balance")?;

    Ok(U256(balance))
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
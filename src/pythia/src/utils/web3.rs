use std::str::FromStr;

use anyhow::{Context, Result};

use candid::Nat;

use ic_cdk::api::management_canister::http_request::{TransformContext, TransformFunc};
use ic_web3_rs::{
    ic::KeyInfo,
    transports::{ic_http_client::CallOptionsBuilder, ICHttp},
    types::{Transaction, TransactionId, TransactionParameters, TransactionReceipt, H256},
    Transport, Web3,
};

use super::{address, canister, nat, time, web3};
use crate::{
    clone_with_state, retry_until_success,
    types::{chains::Chains, errors::PythiaError},
};

const ECDSA_SIGN_CYCLES: u64 = 23_000_000_000;
pub const TRANSFER_GAS_LIMIT: u64 = 21_000;
const TX_SUCCESS_STATUS: u64 = 1;
const TX_WAIT_DELAY: u64 = 3;

pub fn instance(chain_id: &Nat) -> Result<Web3<ICHttp>> {
    Ok(Web3::new(ICHttp::new(&Chains::get(chain_id)?.rpc, None)?))
}

pub async fn get_tx(chain_id: &Nat, tx_hash: &str) -> Result<Transaction> {
    let tx_hash = H256::from_str(tx_hash)?;
    let w3 = instance(chain_id)?;

    let tx_receipt = retry_until_success!(w3
        .eth()
        .transaction_receipt(tx_hash, canister::transform_ctx_tx_with_logs()))?
    .context(PythiaError::TxDoesNotExist)?;

    match tx_receipt.status {
        Some(status) => {
            if status.as_u64() != 1 {
                return Err(PythiaError::TxHasFailed.into());
            }
        }
        None => return Err(PythiaError::TxNotExecuted.into()),
    }

    retry_until_success!(w3
        .eth()
        .transaction(TransactionId::from(tx_hash), canister::transform_ctx_tx()))?
    .context(PythiaError::TxDoesNotExist)
}

pub async fn gas_price(chain_id: &Nat) -> Result<Nat> {
    let w3 = instance(chain_id)?;
    Ok(nat::from_u256(&retry_until_success!(w3
        .eth()
        .gas_price(canister::transform_ctx()))?))
}

#[inline(always)]
pub fn key_info() -> KeyInfo {
    KeyInfo {
        derivation_path: vec![vec![]],
        key_name: clone_with_state!(key_name),
        ecdsa_sign_cycles: Some(ECDSA_SIGN_CYCLES),
    }
}

pub async fn transfer(chain_id: &Nat, to: &str, value: &Nat) -> Result<()> {
    let w3 = instance(chain_id)?;
    let from = canister::pma().await?;
    let from_h160 = address::to_h160(&from)?;
    let to = address::to_h160(to)?;

    let nonce = retry_until_success!(w3.eth().transaction_count(
        from_h160,
        None,
        canister::transform_ctx()
    ))?;
    let mut gas_price = retry_until_success!(w3.eth().gas_price(canister::transform_ctx()))?;

    // multiply the gas_price to 1.2 to avoid long transaction confirmation
    gas_price = (gas_price / 10) * 12;

    let tx = TransactionParameters {
        gas: TRANSFER_GAS_LIMIT.into(),
        gas_price: Some(gas_price),
        to: Some(to),
        value: nat::to_u256(value),
        nonce: Some(nonce),
        ..Default::default()
    };

    let signed_tx = w3
        .accounts()
        .sign_transaction(tx, from, key_info(), nat::to_u64(chain_id))
        .await?;

    let tx_hash = retry_until_success!(w3.eth().send_raw_transaction(
        signed_tx.raw_transaction.clone(),
        canister::transform_ctx_tx()
    ))?;
    web3::wait_for_success_confirmation(&w3, &tx_hash, 60)
        .await
        .context(PythiaError::WaitingForSuccessConfirmationFailed)?;
    Ok(())
}

pub async fn transfer_all(chain_id: &Nat, to: &str) -> Result<()> {
    let w3 = instance(chain_id)?;
    let from = canister::pma().await?;
    let from_h160 = address::to_h160(&from)?;
    let to = address::to_h160(to)?;

    let nonce = retry_until_success!(w3.eth().transaction_count(
        from_h160,
        None,
        canister::transform_ctx()
    ))?;
    let mut gas_price = retry_until_success!(w3.eth().gas_price(canister::transform_ctx()))?;
    // multiply the gas_price to 1.2 to avoid long transaction confirmation
    gas_price = (gas_price / 10) * 12;
    let mut value =
        retry_until_success!(w3.eth().balance(from_h160, None, canister::transform_ctx()))?;
    value -= gas_price * TRANSFER_GAS_LIMIT;

    let tx = TransactionParameters {
        gas: TRANSFER_GAS_LIMIT.into(),
        gas_price: Some(gas_price),
        to: Some(to),
        value,
        nonce: Some(nonce),
        ..Default::default()
    };

    let signed_tx = w3
        .accounts()
        .sign_transaction(tx, from, key_info(), nat::to_u64(chain_id))
        .await?;

    let tx_hash = retry_until_success!(w3
        .eth()
        .send_raw_transaction(signed_tx.raw_transaction.clone(), canister::transform_ctx()))?;
    web3::wait_for_success_confirmation(&w3, &tx_hash, 60)
        .await
        .context(PythiaError::WaitingForSuccessConfirmationFailed)?;

    Ok(())
}

pub async fn wait_for_success_confirmation<T: Transport>(
    w3: &Web3<T>,
    tx_hash: &H256,
    timeout: u64,
) -> Result<TransactionReceipt> {
    let receipt = wait_for_confirmation(w3, tx_hash, timeout).await?;

    let tx_status = receipt.status.expect("tx should be confirmed").as_u64();

    if tx_status != TX_SUCCESS_STATUS {
        return Err(PythiaError::TxHasFailed.into());
    }

    Ok(receipt)
}

pub async fn wait_for_confirmation<T: Transport>(
    w3: &Web3<T>,
    tx_hash: &H256,
    timeout: u64,
) -> Result<TransactionReceipt> {
    let call_opts = CallOptionsBuilder::default()
        .transform(Some(TransformContext {
            function: TransformFunc(candid::Func {
                principal: ic_cdk::api::id(),
                method: "transform".into(),
            }),
            context: vec![],
        }))
        .cycles(None)
        .max_resp(None)
        .build()
        .expect("failed to build call options");

    let end_time = time::in_seconds() + timeout;
    while time::in_seconds() < end_time {
        time::wait(TX_WAIT_DELAY).await;

        let tx_receipt =
            retry_until_success!(w3.eth().transaction_receipt(*tx_hash, call_opts.clone()))
                .context(PythiaError::UnableToGetTxReceipt)?;

        if let Some(tx_receipt) = tx_receipt {
            if tx_receipt.status.is_some() {
                return Ok(tx_receipt);
            }
        }
    }

    Err(PythiaError::TxTimeout.into())
}

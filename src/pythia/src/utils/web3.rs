use std::str::FromStr;

use anyhow::{Context, Result};

use candid::Nat;
use ic_dl_utils::retry_until_success;
use ic_web3::{
    ic::KeyInfo,
    transports::ICHttp,
    types::{Transaction, TransactionId, TransactionParameters, H256},
    Web3,
};

use super::{address, canister, nat};
use crate::{
    clone_with_state,
    types::{chains::Chains, errors::PythiaError},
};

const ECDSA_SIGN_CYCLES: u64 = 23_000_000_000;
const TRANSFER_GAS_LIMIT: u64 = 21_000;

pub fn instance(chain_id: &Nat) -> Result<Web3<ICHttp>> {
    Ok(Web3::new(ICHttp::new(&Chains::get(chain_id)?.rpc, None)?))
}

pub async fn get_tx(chain_id: &Nat, tx_hash: &str) -> Result<Transaction> {
    let tx_hash = H256::from_str(tx_hash)?;
    let w3 = instance(chain_id)?;

    let tx_receipt = retry_until_success!(w3.eth().transaction_receipt(tx_hash))?
        .context(PythiaError::TxDoesNotExist)?;

    match tx_receipt.status {
        Some(status) => {
            if status.as_u64() != 1 {
                return Err(PythiaError::TxHasFailed.into());
            }
        }
        None => return Err(PythiaError::TxNotExecuted.into()),
    }

    retry_until_success!(w3.eth().transaction(TransactionId::from(tx_hash)))?
        .context(PythiaError::TxDoesNotExist)
}

pub async fn gas_price(chain_id: &Nat) -> Result<Nat> {
    let w3 = instance(chain_id)?;
    Ok(nat::from_u256(&retry_until_success!(w3.eth().gas_price())?))
}

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

    let nonce = retry_until_success!(w3.eth().transaction_count(from_h160, None))?;
    let mut gas_price = retry_until_success!(w3.eth().gas_price())?;
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

    let tx_hash = retry_until_success!(w3
        .eth()
        .send_raw_transaction(signed_tx.raw_transaction.clone()))?;
    ic_dl_utils::evm::wait_for_success_confirmation(&w3, &tx_hash, 60)
        .await
        .context(PythiaError::WaitingForSuccessConfirmationFailed)?;
    Ok(())
}

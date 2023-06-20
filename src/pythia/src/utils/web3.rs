use std::str::FromStr;

use anyhow::{Context, Result};

use candid::Nat;
use ic_dl_utils::retry_until_success;
use ic_web3::{
    ic::KeyInfo,
    transports::ICHttp,
    types::{Transaction, TransactionId, H256},
    Web3,
};

use super::nat;
use crate::{
    clone_with_state,
    types::{chains::Chains, errors::PythiaError},
};

const ECDSA_SIGN_CYCLES: u64 = 23_000_000_000;

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

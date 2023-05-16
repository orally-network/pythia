use std::str::FromStr;

use anyhow::{anyhow, Result};

use ic_cdk::export::candid::Nat;
use ic_cdk_macros::{query, update};
use ic_web3::types::H160;

use crate::{utils::validate_caller, Chain, PythiaError, CHAINS, U256};

#[update]
pub fn add_chain(
    chain_id: Nat,
    rpc: String,
    min_balance: Nat,
    treasurer: String,
) -> Result<(), String> {
    _add_chain(chain_id, rpc, min_balance, treasurer).map_err(|e| e.to_string())
}

fn _add_chain(chain_id: Nat, rpc: String, min_balance: Nat, treasurer: String) -> Result<()> {
    validate_caller()?;

    let treasurer = H160::from_str(&treasurer)?;

    let chain = Chain::new(&chain_id, &rpc, &min_balance, &treasurer)?;

    CHAINS.with(|chains| {
        let mut chains = chains.borrow_mut();
        if chains.contains_key(&chain.chain_id) {
            return Err(anyhow!(PythiaError::ChainAlreadyExists));
        };

        chains.insert(chain.chain_id, chain);

        Ok(())
    })
}

#[update]
pub fn remove_chain(chain_id: Nat) -> Result<(), String> {
    _remove_chain(chain_id).map_err(|e| e.to_string())
}

fn _remove_chain(chain_id: Nat) -> Result<()> {
    validate_caller()?;

    CHAINS.with(|chains| {
        let mut chains = chains.borrow_mut();
        chains
            .remove(&U256::from(chain_id))
            .ok_or(anyhow!(PythiaError::ChainDoesNotExist))?;

        Ok(())
    })
}

#[update]
pub fn update_chain_rpc(chain_id: Nat, rpc: String) -> Result<(), String> {
    _update_chain_rpc(chain_id, rpc).map_err(|e| e.to_string())
}

fn _update_chain_rpc(chain_id: Nat, rpc: String) -> Result<()> {
    validate_caller()?;

    CHAINS.with(|chains| {
        let mut chains = chains.borrow_mut();
        let chain = chains
            .get_mut(&U256::from(chain_id))
            .ok_or(PythiaError::ChainDoesNotExist)?;

        chain.rpc = rpc.parse()?;

        Ok(())
    })
}

#[update]
pub fn update_chain_min_balance(chain_id: Nat, min_balance: Nat) -> Result<(), String> {
    _update_chain_min_balance(chain_id, min_balance).map_err(|e| e.to_string())
}

fn _update_chain_min_balance(chain_id: Nat, min_balance: Nat) -> Result<()> {
    validate_caller()?;

    CHAINS.with(|chains| {
        let mut chains = chains.borrow_mut();
        let chain = chains
            .get_mut(&U256::from(chain_id))
            .ok_or(PythiaError::ChainDoesNotExist)?;

        chain.min_balance = U256::from(min_balance);

        Ok(())
    })
}

#[query]
pub fn get_chain_rpc(chain_id: Nat) -> Result<String, String> {
    _get_chain_rpc(chain_id).map_err(|e| e.to_string())
}

fn _get_chain_rpc(chain_id: Nat) -> Result<String> {
    validate_caller()?;

    CHAINS.with(|chains| {
        let chains = chains.borrow();
        let chain = chains
            .get(&U256::from(chain_id))
            .ok_or(PythiaError::ChainDoesNotExist)?;

        Ok(chain.rpc.to_string())
    })
}

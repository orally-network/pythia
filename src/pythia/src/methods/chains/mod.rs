use std::{str::FromStr, collections::HashMap};

use anyhow::{anyhow, Result};

use ic_cdk::export::candid::Nat;
use ic_cdk_macros::{query, update};
use ic_utils::logger::log_message;
use ic_web3::types::H160;

use crate::{
    types::chains::CandidTypeChain, utils::validate_caller, Chain, PythiaError, STATE,
};

#[update]
pub fn add_chain(
    chain_id: Nat,
    rpc: String,
    min_balance: Nat,
    treasurer: String,
    block_gas_limit: Nat,
) -> Result<(), String> {
    _add_chain(chain_id, rpc, min_balance, treasurer, block_gas_limit).map_err(|e| format!("{e:?}"))
}

fn _add_chain(
    chain_id: Nat,
    rpc: String,
    min_balance: Nat,
    treasurer: String,
    block_gas_limit: Nat,
) -> Result<()> {
    validate_caller()?;
    let treasurer = H160::from_str(&treasurer)?;

    let chain = Chain::new(
        &chain_id,
        &rpc,
        &min_balance,
        &treasurer,
        &block_gas_limit,
    )?;

    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.chains.contains_key(&chain.chain_id) {
            return Err(anyhow!(PythiaError::ChainAlreadyExists));
        };

        state.chains.insert(chain.chain_id.clone(), chain.clone());
        state.balances.insert(chain.chain_id.clone(), HashMap::new());
        state.subscriptions.insert(chain.chain_id.clone(), Vec::new());
        state.withdraw_requests.insert(chain.chain_id.clone(), Vec::new());

        log_message(format!("[CHAIN ID: {}] creation", chain_id.0));

        Ok(())
    })
}

#[update]
pub fn remove_chain(chain_id: Nat) -> Result<(), String> {
    _remove_chain(chain_id).map_err(|e| format!("{e:?}"))
}

fn _remove_chain(chain_id: Nat) -> Result<()> {
    validate_caller()?;

    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.chains
            .remove(&chain_id)
            .ok_or(anyhow!(PythiaError::ChainDoesNotExist))?;

        log_message(format!("[CHAIN ID: {}] removing", chain_id.0));

        Ok(())
    })
}

#[update]
pub fn update_chain_rpc(chain_id: Nat, rpc: String) -> Result<(), String> {
    _update_chain_rpc(chain_id, rpc).map_err(|e| format!("{e:?}"))
}

fn _update_chain_rpc(chain_id: Nat, rpc: String) -> Result<()> {
    validate_caller()?;

    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let chain = state.chains
            .get_mut(&chain_id)
            .ok_or(PythiaError::ChainDoesNotExist)?;

        chain.rpc = rpc.parse()?;

        log_message(format!("[CHAIN ID: {}] updating rpc: {}", chain_id.0, rpc));

        Ok(())
    })
}

#[update]
pub fn update_chain_min_balance(chain_id: Nat, min_balance: Nat) -> Result<(), String> {
    _update_chain_min_balance(chain_id, min_balance).map_err(|e| format!("{e:?}"))
}

fn _update_chain_min_balance(chain_id: Nat, min_balance: Nat) -> Result<()> {
    validate_caller()?;

    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let chain = state.chains
            .get_mut(&chain_id)
            .ok_or(PythiaError::ChainDoesNotExist)?;

        chain.min_balance = min_balance;

        log_message(format!(
            "[CHAIN ID: {}] updating min balance: {}",
            chain_id.0, chain.min_balance.0
        ));

        Ok(())
    })
}

#[query]
pub fn get_chain_rpc(chain_id: Nat) -> Result<String, String> {
    _get_chain_rpc(chain_id).map_err(|e| format!("{e:?}"))
}

fn _get_chain_rpc(chain_id: Nat) -> Result<String> {
    validate_caller()?;

    STATE.with(|state| {
        let state = state.borrow_mut();
        let chain = state.chains
            .get(&chain_id)
            .ok_or(PythiaError::ChainDoesNotExist)?;

        Ok(chain.rpc.to_string())
    })
}

#[query]
pub fn get_chains() -> Result<Vec<CandidTypeChain>, String> {
    Ok(STATE.with(|state| {
        state
            .borrow()
            .chains
            .values()
            .cloned()
            .map(|e| CandidTypeChain {
                chain_id: e.chain_id.into(),
                rpc: e.rpc,
                min_balance: e.min_balance.into(),
                treasurer: hex::encode(e.treasurer.as_bytes()),
            })
            .collect()
    }))
}

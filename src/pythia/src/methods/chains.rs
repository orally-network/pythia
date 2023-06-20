use anyhow::{anyhow, Result, Context};

use ic_cdk::export::candid::Nat;
use ic_cdk_macros::{query, update};
use ic_utils::logger::log_message;

use crate::{utils::validator, Chain, PythiaError, types::{chains::{Chains, ChainUpdator}, balance::Balances, subscription::Subscriptions, withdraw::WithdrawRequests}};

/// Add a new chain to the state.
/// 
/// # Arguments
/// 
/// * `chain_id` - Unique identifier of the chain, for example Ethereum Mainnet is 1
/// * `rpc` - RPC endpoint.
/// * `min_balance` - Minimum balance, used to check if balances have sufficient funds.
/// * `block_gas_limit` - Max gas limit per block.
/// 
/// # Returns
/// 
/// Returns a result that can contain an error message
#[update]
pub fn add_chain(chain_id: Nat, rpc: String, min_balance: Nat, block_gas_limit: Nat) -> Result<(), String> {
    _add_chain(chain_id, rpc, min_balance, block_gas_limit)
        .map_err(|e| format!("failed to add a chain: {e:?}"))
}

fn _add_chain(chain_id: Nat, rpc: String, min_balance: Nat, block_gas_limit: Nat) -> Result<()> {
    validator::caller()?;
    if Chains::is_exists(&chain_id) {
        return Err(anyhow!(PythiaError::ChainAlreadyExists));
    }

    Chains::add(&chain_id, &rpc, &min_balance, &block_gas_limit)
        .context(PythiaError::UnableToAddNewChain)?;

    Balances::init_new_chain(&chain_id)?;
    Subscriptions::init_new_chain(&chain_id)?;
    WithdrawRequests::init_new_chain(&chain_id)?;

    log_message(format!("[CHAINS] added, id: {chain_id}"));
    Ok(())
}

/// Remove a chain from the state.
/// 
/// # Arguments
/// 
/// * `chain_id` - Unique identifier of the chain, for example Ethereum Mainnet is 1
/// 
/// # Returns
/// 
/// Returns a result that can contain an error message
#[update]
pub fn remove_chain(chain_id: Nat) -> Result<(), String> {
    _remove_chain(chain_id)
        .map_err(|e| format!("failed to remove a chain: {e:?}"))
}

fn _remove_chain(chain_id: Nat) -> Result<()> {
    validator::caller()?;
    Chains::remove(&chain_id)
        .context(PythiaError::UnableToRemoveChain)?;
    
    Balances::deinit_chain(&chain_id)
        .context(PythiaError::UnableToRemoveChain)?;
    Subscriptions::deinit_chain(&chain_id)
        .context(PythiaError::UnableToRemoveChain)?;
    WithdrawRequests::deinit_chain(&chain_id)
        .context(PythiaError::UnableToRemoveChain)?;

    log_message(format!("[CHAINS] removed, id: {chain_id}"));
    Ok(())
}

/// Update a chain RPC in the state.
/// 
/// # Arguments
/// 
/// * `chain_id` - Unique identifier of the chain, for example Ethereum Mainnet is 1
/// * `rpc` - RPC endpoint.
/// 
/// # Returns
/// 
/// Returns a result that can contain an error message
#[update]
pub fn update_chain_rpc(chain_id: Nat, rpc: String) -> Result<(), String> {
    _update_chain_rpc(chain_id, rpc)
        .map_err(|e| format!("failed to update a chain RPC: {e:?}"))
}

fn _update_chain_rpc(chain_id: Nat, rpc: String) -> Result<()> {
    validator::caller()?;
    Chains::update(&chain_id, ChainUpdator {
        rpc: Some(rpc.clone()),
        ..Default::default()
    }).context(PythiaError::UnableToUpdateChain)?;
    
    log_message(format!("[CHAINS] RPC updated: {rpc}, id: {chain_id}"));
    Ok(())
}

/// Update a chain minimum balance in the state.
/// 
/// # Arguments
/// 
/// * `chain_id` - Unique identifier of the chain, for example Ethereum Mainnet is 1
/// * `min_balance` - Minimum balance, used to check if balances have sufficient funds.
/// 
/// # Returns
/// 
/// Returns a result that can contain an error message
#[update]
pub fn update_chain_min_balance(chain_id: Nat, min_balance: Nat) -> Result<(), String> {
    _update_chain_min_balance(chain_id, min_balance)
        .map_err(|e| format!("failed to update a chain minimum balance: {e:?}"))
}

fn _update_chain_min_balance(chain_id: Nat, min_balance: Nat) -> Result<()> {
    validator::caller()?;
    Chains::update(&chain_id, ChainUpdator {
        min_balance: Some(min_balance.clone()),
        ..Default::default()
    }).context(PythiaError::UnableToUpdateChain)?;

    log_message(format!("[CHAINS] minimum balance updated: {min_balance}, id: {chain_id}"));
    Ok(())
}

/// Get a chain RPC from the state.
/// 
/// # Arguments
/// 
/// * `chain_id` - Unique identifier of the chain, for example Ethereum Mainnet is 1
/// 
/// # Returns
/// 
/// Returns a GetChainRPCResponse that can contain an error message
#[query]
pub fn get_chain_rpc(chain_id: Nat) -> Result<String, String> {
    _get_chain_rpc(chain_id)
        .map_err(|e| format!("failed to get a chain RPC{e:?}"))
}

fn _get_chain_rpc(chain_id: Nat) -> Result<String> {
    validator::caller()?;
    Chains::get_rpc(&chain_id)
        .context(PythiaError::UnableToGetChainRPC)
}

/// Get all chains from the state.
/// 
/// # Returns
/// 
/// Returns a vector that contains chains
#[query]
pub fn get_chains() -> Vec<Chain> {
    Chains::get_all()
}

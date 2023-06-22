use anyhow::{anyhow, Context, Result};

use ic_cdk::export::candid::Nat;
use ic_cdk_macros::{query, update};

use crate::{
    log,
    types::{
        balance::Balances,
        chains::{ChainUpdator, Chains, CreateChainRequest},
        subscription::Subscriptions,
        withdraw::WithdrawRequests,
    },
    utils::{canister, validator},
    Chain, PythiaError,
};

/// Add a new chain to the state.
///
/// # Arguments
///
/// * `req` - the CreateChainRequest
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
pub async fn add_chain(req: CreateChainRequest) -> Result<(), String> {
    _add_chain(req)
        .await
        .map_err(|e| format!("failed to add a chain: {e:?}"))
}

async fn _add_chain(req: CreateChainRequest) -> Result<()> {
    validator::caller()?;
    if Chains::is_exists(&req.chain_id) {
        return Err(anyhow!(PythiaError::ChainAlreadyExists));
    }

    Chains::add(&req).context(PythiaError::UnableToAddNewChain)?;

    Balances::init_new_chain(&req.chain_id).context(PythiaError::UnableToAddNewChain)?;
    Subscriptions::init_new_chain(&req.chain_id).context(PythiaError::UnableToAddNewChain)?;
    WithdrawRequests::init_new_chain(&req.chain_id).context(PythiaError::UnableToAddNewChain)?;

    let pma = canister::pma().await.context(PythiaError::UnableToGetPMA)?;
    Balances::create(&req.chain_id, &pma).context(PythiaError::UnableToAddNewBalance)?;

    log!("[CHAINS] added, id: {}", req.chain_id);
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
    _remove_chain(chain_id).map_err(|e| format!("failed to remove a chain: {e:?}"))
}

fn _remove_chain(chain_id: Nat) -> Result<()> {
    validator::caller()?;
    Chains::remove(&chain_id).context(PythiaError::UnableToRemoveChain)?;

    Balances::deinit_chain(&chain_id).context(PythiaError::UnableToRemoveChain)?;
    Subscriptions::deinit_chain(&chain_id).context(PythiaError::UnableToRemoveChain)?;
    WithdrawRequests::deinit_chain(&chain_id).context(PythiaError::UnableToRemoveChain)?;

    log!("[CHAINS] removed, id: {chain_id}");
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
    _update_chain_rpc(chain_id, rpc).map_err(|e| format!("failed to update a chain RPC: {e:?}"))
}

fn _update_chain_rpc(chain_id: Nat, rpc: String) -> Result<()> {
    validator::caller()?;
    Chains::update(
        &chain_id,
        ChainUpdator {
            rpc: Some(rpc.clone()),
            ..Default::default()
        },
    )
    .context(PythiaError::UnableToUpdateChain)?;

    log!("[CHAINS] RPC updated: {rpc}, id: {chain_id}");
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
    Chains::update(
        &chain_id,
        ChainUpdator {
            min_balance: Some(min_balance.clone()),
            ..Default::default()
        },
    )
    .context(PythiaError::UnableToUpdateChain)?;

    log!("[CHAINS] minimum balance updated: {min_balance}, id: {chain_id}");
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
    _get_chain_rpc(chain_id).map_err(|e| format!("failed to get a chain RPC{e:?}"))
}

fn _get_chain_rpc(chain_id: Nat) -> Result<String> {
    validator::caller()?;
    Chains::get_rpc(&chain_id).context(PythiaError::UnableToGetChainRPC)
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

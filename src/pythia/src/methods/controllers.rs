use anyhow::{Context, Result};
use candid::Nat;
use ic_cdk::{query, update};

use crate::{
    jobs::{publisher, withdraw},
    log,
    types::{balance::Balances, logger::CONTROLLERS, state::State, timer::Timer},
    update_state,
    utils::{address, canister, validator, web3},
    PythiaError, STATE,
};

/// Update the tx fee.
///
/// # Arguments
///
/// * `tx_fee` - New tx fee, used for collecting fee from balances.
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
pub fn update_tx_fee(tx_fee: Nat) -> Result<(), String> {
    _update_tx_fee(tx_fee).map_err(|e| format!("failed to update the tx fee{e:?}"))
}

#[inline]
pub fn _update_tx_fee(tx_fee: Nat) -> Result<()> {
    validator::caller()?;
    update_state!(tx_fee, tx_fee.clone());
    log!("[{CONTROLLERS}] tx fee updated: {tx_fee}");
    Ok(())
}

/// Get the current state.
///
/// # Returns
///
/// Returns the current state
#[query]
pub fn get_cfg() -> State {
    _get_cfg()
}

#[inline]
pub fn _get_cfg() -> State {
    STATE.with(|state| state.borrow().clone())
}

/// Update the subscriptions limit for a wallet.
///
/// # Arguments
///
/// * `limit` - New subs limit for a wallet, used to check if there is subscriptions for a waller overflow.
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
pub fn update_subs_limit_wallet(limit: Nat) -> Result<(), String> {
    _update_subs_limit_wallet(limit)
        .map_err(|e| format!("failed to update the subscriptions limit for a wallet: {e:?}"))
}

#[inline]
fn _update_subs_limit_wallet(limit: Nat) -> Result<()> {
    validator::caller()?;
    update_state!(subs_limit_wallet, limit.clone());
    log!("[{CONTROLLERS}] subscriptions limit for a wallet updated: {limit}");
    Ok(())
}

/// Update the subscriptions limit total.
///
/// # Arguments
///
/// * `limit` - New subs limit total, used to check if there is subscriptions overflow.
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
pub fn update_subs_limit_total(limit: Nat) -> Result<(), String> {
    _update_subs_limit_total(limit)
        .map_err(|e| format!("failed to update the subscriptions limit: {e:?}"))
}

#[inline]
fn _update_subs_limit_total(limit: Nat) -> Result<()> {
    validator::caller()?;
    update_state!(subs_limit_total, limit.clone());
    log!("[{CONTROLLERS}] subscriptions limit: {limit}");
    Ok(())
}

/// Update the timer frequency.
///
/// # Arguments
///
/// * `frequency` - New timer frequency, when will a new timer will be executed.
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
pub fn update_timer_frequency(frequency: Nat) -> Result<(), String> {
    _update_timer_frequency(frequency)
        .map_err(|e| format!("failed to update the timer frequency: {e:?}"))
}

#[inline]
fn _update_timer_frequency(frequency: Nat) -> Result<()> {
    validator::caller()?;
    update_state!(timer_frequency, frequency.clone());
    log!("[{CONTROLLERS}] the timer frequency updated: {frequency}");
    Ok(())
}

/// Execute the withdraw job
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
pub fn execute_withdraw_job() -> Result<(), String> {
    _execute_withdraw_job().map_err(|e| format!("failed to execute the withdraw job: {e:?}"))
}

#[inline]
fn _execute_withdraw_job() -> Result<()> {
    validator::caller()?;
    withdraw::execute();
    log!("[{CONTROLLERS}] withdraw job forcefully executed");
    Ok(())
}

/// Execute the publisher job
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
pub fn execute_publisher_job() -> Result<(), String> {
    _execute_publisher_job().map_err(|e| format!("failed to execute the publisher job: {e:?}"))
}

#[inline]
fn _execute_publisher_job() -> Result<()> {
    validator::caller()?;
    publisher::execute();
    log!("[{CONTROLLERS}] publisher job forcefully executed");
    Ok(())
}

/// Withdraw the platform fees.
///
/// # Arguments
///
/// * `chain_id` - Unique identifier of the chain, for example Ethereum Mainnet is 1
/// * `receiver` - Address of the receiver
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
pub async fn withdraw_fee(chain_id: Nat, receiver: String) -> Result<(), String> {
    _withdraw_fee(chain_id, receiver)
        .await
        .map_err(|e| format!("failed to withdraw the fee: {e:?}"))
}

#[inline]
async fn _withdraw_fee(chain_id: Nat, receiver: String) -> Result<()> {
    validator::caller()?;
    let receiver = address::normalize(&receiver).context(PythiaError::InvalidAddressFormat)?;
    let pma = canister::pma().await.context(PythiaError::UnableToGetPMA)?;
    let value = Balances::get(&chain_id, &pma).context(PythiaError::UnableToGetBalance)?;
    web3::transfer(&chain_id, &receiver, &value)
        .await
        .context(PythiaError::UnableToTransferFunds)?;
    Balances::reduce(&chain_id, &pma, &value).context(PythiaError::UnableToReduceBalance)?;

    log!("[] fees were withdrawn to: {receiver}");
    Ok(())
}

/// Withdraw all the balance.
///
/// # Arguments
///
/// * `chain_id` - Unique identifier of the chain, for example Ethereum Mainnet is 1
/// * `receiver` - Address of the receiver
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
pub async fn withdraw_all_balance(chain_id: Nat, receiver: String) -> Result<(), String> {
    _withdraw_all_balance(chain_id, receiver)
        .await
        .map_err(|e| format!("failed to withdraw all balance: {e:?}"))
}

#[inline]
async fn _withdraw_all_balance(chain_id: Nat, receiver: String) -> Result<()> {
    validator::caller()?;
    let receiver = address::normalize(&receiver).context(PythiaError::InvalidAddressFormat)?;

    web3::transfer_all(&chain_id, &receiver)
        .await
        .context(PythiaError::UnableToTransferFunds)?;

    log!("[{CONTROLLERS}] all balance was withdrawn to: {receiver}");
    Ok(())
}

/// Stop main timer
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
pub fn stop_timer() -> Result<(), String> {
    _stop_timer().map_err(|e| format!("failed to stop the timer: {e:?}"))
}

#[inline]
fn _stop_timer() -> Result<()> {
    validator::caller()?;
    Timer::deactivate().context(PythiaError::UnableToDeactivateTimer)?;
    log!("[{CONTROLLERS}] timer was stopped");
    Ok(())
}

#[update]
pub fn clear_balance(chain_id: Nat, address: String) -> Result<(), String> {
    _clear_balance(chain_id, address).map_err(|e| format!("failed to clear the balance: {e:?}"))
}

#[inline]
fn _clear_balance(chain_id: Nat, address: String) -> Result<()> {
    validator::caller()?;
    let address = address::normalize(&address).context(PythiaError::InvalidAddressFormat)?;
    Balances::clear(&chain_id, &address).context(PythiaError::UnableToClearBalance)?;
    log!("[{CONTROLLERS}] balance was cleared for: {address}");
    Ok(())
}

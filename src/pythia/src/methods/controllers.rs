use anyhow::{Context, Result};

use ic_cdk::export::candid::Nat;
use ic_cdk_macros::update;
use ic_utils::logger::log_message;

use crate::{
    clone_with_state,
    jobs::{publisher, withdraw},
    log,
    types::{balance::Balances, timer::Timer},
    update_state,
    utils::{address, canister, validator, web3},
    PythiaError,
};

/// Update the controllers.
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
pub async fn update_controllers() -> Result<(), String> {
    _update_controllers()
        .await
        .map_err(|e| format!("failed to update the conrollers: {e:?}"))
}

async fn _update_controllers() -> Result<()> {
    if clone_with_state!(initialized) {
        validator::caller()?;
    } else {
        update_state!(initialized, true);
    }
    let controllers = canister::get_controllers()
        .await
        .context(PythiaError::UnableToGetControllers)?;
    update_state!(controllers, controllers.clone());

    log!("[CONTROLLERS] updated: {controllers:?}");
    Ok(())
}

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

pub fn _update_tx_fee(tx_fee: Nat) -> Result<()> {
    validator::caller()?;
    update_state!(tx_fee, tx_fee.clone());
    log_message(format!("[CONTROLLERS] tx fee updated: {tx_fee}"));
    Ok(())
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

fn _update_subs_limit_wallet(limit: Nat) -> Result<()> {
    validator::caller()?;
    update_state!(subs_limit_wallet, limit.clone());
    log!("[CONTROLLERS] subscriptions limit for a wallet updated: {limit}");
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

fn _update_subs_limit_total(limit: Nat) -> Result<()> {
    validator::caller()?;
    update_state!(subs_limit_total, limit.clone());
    log!("[CONTROLLERS] subscriptions limit: {limit}");
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

fn _update_timer_frequency(frequency: Nat) -> Result<()> {
    validator::caller()?;
    update_state!(timer_frequency, frequency.clone());
    log!("[CONTROLLERS] the timer frequency updated: {frequency}");
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

fn _execute_withdraw_job() -> Result<()> {
    validator::caller()?;
    withdraw::execute();
    log!("[CONTROLLERS] withdraw job forcefully executed");
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

fn _execute_publisher_job() -> Result<()> {
    validator::caller()?;
    publisher::execute();
    log!("[CONTROLLERS] publisher job forcefully executed");
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

async fn _withdraw_fee(chain_id: Nat, receiver: String) -> Result<()> {
    validator::caller()?;
    let receiver = address::normalize(&receiver)
        .context(PythiaError::InvalidAddressFormat)?;
    let pma = canister::pma()
        .await
        .context(PythiaError::UnableToGetPMA)?;
    let value = Balances::get(&chain_id, &pma)
        .context(PythiaError::UnableToGetBalance)?;
    web3::transfer(&chain_id, &receiver, &value)
        .await
        .context(PythiaError::UnableToTransferFunds)?;
    Balances::reduce(&chain_id, &pma, &value)
        .context(PythiaError::UnableToReduceBalance)?;

    log!("[CONTROLLERS] fees were withdrawn to: {receiver}");
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

async fn _withdraw_all_balance(chain_id: Nat, receiver: String) -> Result<()> {
    validator::caller()?;
    let receiver = address::normalize(&receiver)
        .context(PythiaError::InvalidAddressFormat)?;
    
    web3::transfer_all(&chain_id, &receiver)
        .await
        .context(PythiaError::UnableToTransferFunds)?;

    log!("[CONTROLLERS] all balance was withdrawn to: {receiver}");
    Ok(())
}

/// Stop main timer
/// 
/// # Returns
/// 
/// Returns a result that can contain an error message
#[update]
pub fn stop_timer() -> Result<(), String> {
    _stop_timer()
        .map_err(|e| format!("failed to stop the timer: {e:?}"))
}

fn _stop_timer() -> Result<()> {
    validator::caller()?;
    Timer::deactivate()
        .context(PythiaError::UnableToDeactivateTimer)?;
    log!("[CONTROLLERS] timer was stopped");
    Ok(())
}

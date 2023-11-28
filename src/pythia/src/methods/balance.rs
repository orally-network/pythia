use anyhow::{Context, Result};
use candid::Nat;
use ic_cdk::{query, update};

use crate::{
    jobs::withdraw,
    log,
    types::{
        balance::Balances, errors::PythiaError, subscription::Subscriptions, timer::Timer,
        whitelist, withdraw::WithdrawRequests,
    },
    utils::{address, canister, nat, siwe, web3},
};

/// Get the PMA address
///
/// # Returns
///
/// Returns a result with the PMA address
#[update]
pub async fn get_pma() -> Result<String, String> {
    crate::utils::canister::pma()
        .await
        .map_err(|e| format!("failed to get the PMA: {e:?}"))
}

/// Deposit amount to the PMA
///
/// # Arguments
///
/// * `tx_hash` - 256-bit hash of the transaction, for example 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef
/// * `chain_id` - Unique identifier of the chain, for example Ethereum Mainnet is 1
/// * `msg` - SIWE message, For more information, refer to the [SIWE message specification](https://eips.ethereum.org/EIPS/eip-4361)
/// * `sig` - SIWE signature, For more information, refer to the [SIWE message specification](https://eips.ethereum.org/EIPS/eip-4361)
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
pub async fn deposit(
    chain_id: Nat,
    tx_hash: String,
    msg: String,
    sig: String,
) -> Result<(), String> {
    _deposit(chain_id, tx_hash, msg, sig)
        .await
        .map_err(|e| format!("failed to deposit: {e:?}"))
}

#[inline]
async fn _deposit(chain_id: Nat, tx_hash: String, msg: String, sig: String) -> Result<()> {
    let address = siwe::recover(&msg, &sig)
        .await
        .context(PythiaError::UnableToRecoverAddress)?;
    if !whitelist::is_whitelisted(&address) {
        return Err(PythiaError::UserIsNotWhitelisted.into());
    }
    if !Balances::is_exists(&chain_id, &address)? {
        Balances::create(&chain_id, &address).context(PythiaError::UnableToAddNewBalance)?;
    }

    let tx = web3::get_tx(&chain_id, &tx_hash)
        .await
        .context(PythiaError::UnableToGetTx)?;

    let receiver = tx.to.context(PythiaError::TxWithoutReceiver)?;
    let pma = canister::pma_h160()
        .await
        .context(PythiaError::UnableToGetPMA)?;

    if receiver != pma {
        return Err(PythiaError::TxWasNotSentToPma.into());
    }

    Balances::save_nonce(&chain_id, &address, &nat::from_u256(&tx.nonce))
        .context(PythiaError::UnableToSaveNonce)?;

    let amount = nat::from_u256(&tx.value);
    #[allow(clippy::cmp_owned)]
    if amount <= Nat::from(0) {
        return Ok(());
    }

    Balances::add_amount(&chain_id, &address, &amount)
        .context(PythiaError::UnableToIncreaseBalance)?;

    log!("[{address}] deposited amount {amount}");
    Ok(())
}

/// Withdraw amount from the PMA
///
/// # Arguments
///
/// * `chain_id` - Unique identifier of the chain, for example Ethereum Mainnet is 1
/// * `receiver` - Address of the receiver, for example 0x1234567890abcdef1234567890abcdef12345678
/// * `msg` - SIWE message, For more information, refer to the [SIWE message specification](https://eips.ethereum.org/EIPS/eip-4361)
/// * `sig` - SIWE signature, For more information, refer to the [SIWE message specification](https://eips.ethereum.org/EIPS/eip-4361)
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
pub async fn withdraw(
    chain_id: Nat,
    receiver: String,
    msg: String,
    sig: String,
) -> Result<(), String> {
    _withdraw(chain_id, receiver, msg, sig)
        .await
        .map_err(|e| format!("failed to withdraw: {e:?}"))
}

#[inline]
async fn _withdraw(chain_id: Nat, msg: String, sig: String, receiver: String) -> Result<()> {
    let address = siwe::recover(&msg, &sig)
        .await
        .context(PythiaError::UnableToRecoverAddress)?;
    let gas_price = web3::gas_price(&chain_id)
        .await
        .context(PythiaError::UnableToGetGasPrice)?;
    let amount = Balances::get_value_for_witndraw(&chain_id, &address, &gas_price)
        .context(PythiaError::UnableToGetValueForWithdraw)?;
    Subscriptions::stop_all(Some(chain_id.clone()), vec![], Some(address.clone()))
        .context(PythiaError::UnableToStopSubscriptions)?;
    WithdrawRequests::add(&chain_id, &receiver, &amount)
        .context(PythiaError::UnableToAddWithdrawRequest)?;

    if !Timer::is_active() {
        withdraw::execute();
    }

    log!("[{address}] withdrawed amount {amount}");
    Ok(())
}

/// Get balance of the user
///
/// # Arguments
/// * `chain_id` - Unique identifier of the chain, for example Ethereum Mainnet is 1
/// * `address` - Address of the user, for example 0x1234567890abcdef1234567890abcdef12345678
///
/// # Returns
///
/// Returns a result with address's balance
#[query]
pub fn get_balance(chain_id: Nat, address: String) -> Result<Nat, String> {
    _get_balance(chain_id, address).map_err(|e| format!("failed to get balance: {e:?}"))
}

#[inline]
fn _get_balance(chain_id: Nat, address: String) -> Result<Nat> {
    let address = address::normalize(&address).context(PythiaError::InvalidAddressFormat)?;
    Ok(Balances::get(&chain_id, &address).unwrap_or_default())
}

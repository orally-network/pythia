use ic_cdk::{query, update};

use anyhow::{Context, Result};

use crate::{
    log,
    types::{
        errors::PythiaError,
        subscription::Subscriptions,
        whitelist::{self, Whitelist},
    },
    utils::{address, validator},
};

/// Add an address to the whitelist
///
/// # Arguments
///
/// * `address` - Address to add to the whitelist
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
fn add_to_whitelist(address: String) -> Result<(), String> {
    _add_to_whitelist(address).map_err(|e| format!("failed to add to the whitelist: {e:?}"))
}

fn _add_to_whitelist(address: String) -> Result<()> {
    validator::caller()?;

    let address = address::normalize(&address).context(PythiaError::InvalidAddressFormat)?;
    whitelist::add(&address);

    log!("[WHITELIST] address added to the whitelist: {address}");
    Ok(())
}

/// Remove an address from the whitelist
///
/// # Arguments
///
/// * `address` - Address to remove from the whitelist
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
fn remove_from_whitelist(address: String) -> Result<(), String> {
    _remove_from_whitelist(address)
        .map_err(|e| format!("failed to remove from the whitelist: {e:?}"))
}

fn _remove_from_whitelist(address: String) -> Result<()> {
    validator::caller()?;

    let address = address::normalize(&address).context(PythiaError::InvalidAddressFormat)?;
    whitelist::remove(&address);
    Subscriptions::remove_all(None, vec![], Some(address.clone()))
        .context(PythiaError::UnableToRemoveSubscriptions)?;

    log!("[WHITELIST] address removed from the whitelist: {address}");
    Ok(())
}

/// Blacklist an address
///
/// # Arguments
///
/// * `address` - Address to blacklist
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
fn blacklist(address: String) -> Result<(), String> {
    _blacklist(address).map_err(|e| format!("failed to blacklist user: {e:?}"))
}

fn _blacklist(address: String) -> Result<()> {
    validator::caller()?;

    let address = address::normalize(&address).context(PythiaError::InvalidAddressFormat)?;
    whitelist::blacklist(&address);
    Subscriptions::stop_all(None, vec![], Some(address.clone()))
        .context(PythiaError::UnableToStopSubscriptions)?;

    log!("[WHITELIST] address blacklisted: {address}");
    Ok(())
}

/// Unblacklist an address
///
/// # Arguments
///
/// * `address` - Address to unblacklist
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
fn unblacklist(address: String) -> Result<(), String> {
    _unblacklist(address).map_err(|e| format!("failed to unblacklist user: {e:?}"))
}

fn _unblacklist(address: String) -> Result<()> {
    validator::caller()?;

    let address = address::normalize(&address).context(PythiaError::InvalidAddressFormat)?;
    whitelist::unblacklist(&address);

    log!("[WHITELIST] address unblacklisted: {address}");
    Ok(())
}

/// Check if an address is whitelisted
///
/// # Arguments
///
/// * `address` - Address to check
///
/// # Returns
///
/// Returns the IsWhitelistedResponse canid type
#[query]
fn is_whitelisted(address: String) -> Result<bool, String> {
    _is_whitelisted(address)
        .map_err(|e| format!("failed to check if address is whitelisted: {e:?}"))
}

fn _is_whitelisted(address: String) -> Result<bool> {
    let address = address::normalize(&address).context(PythiaError::InvalidAddressFormat)?;
    Ok(whitelist::is_whitelisted(&address))
}

/// Get the whitelist
///
/// # Returns
///
/// Returns the GetWhiteListResponse
#[query]
fn get_whitelist() -> Result<Whitelist, String> {
    _get_whitelist().map_err(|e| format!("failed to get the whitelist: {e:?}"))
}

fn _get_whitelist() -> Result<Whitelist> {
    validator::caller()?;
    Ok(whitelist::get_list())
}

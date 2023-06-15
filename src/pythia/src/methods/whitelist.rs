use ic_cdk::update;

use anyhow::Result;

use crate::{
    types::whitelist::{self, Whitelist},
    utils::validate_caller,
    STATE,
};

#[update]
fn add_to_whitelist(address: String) -> Result<(), String> {
    _add_to_whitelist(address).map_err(|e| format!("Error adding to whitelist: {}", e))
}

fn _add_to_whitelist(address: String) -> Result<()> {
    validate_caller()?;
    whitelist::add(&address);
    Ok(())
}

#[update]
fn remove_from_whitelist(address: String) -> Result<(), String> {
    _remove_from_whitelist(address).map_err(|e| format!("Error removing from whitelist: {}", e))
}

fn _remove_from_whitelist(address: String) -> Result<()> {
    validate_caller()?;
    whitelist::remove(&address);
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.balances.iter_mut().for_each(|(_, balances)| {
            balances.remove(&address);
        });
        state
            .subscriptions
            .iter_mut()
            .for_each(|(_, subscriptions)| {
                subscriptions.retain(|sub| sub.owner != address);
            });
    });
    Ok(())
}

#[update]
fn blacklist(address: String) -> Result<(), String> {
    _blacklist(address).map_err(|e| format!("Error blacklisting: {}", e))
}

fn _blacklist(address: String) -> Result<()> {
    validate_caller()?;
    whitelist::blacklist(&address);
    STATE.with(|state| {
        state
            .borrow_mut()
            .subscriptions
            .iter_mut()
            .for_each(|(_, subscriptions)| {
                subscriptions.iter_mut().for_each(|sub| {
                    if sub.owner == address {
                        sub.status.is_active = false;
                    }
                })
            });
    });
    Ok(())
}

#[update]
fn unblacklist(address: String) -> Result<(), String> {
    _unblacklist(address).map_err(|e| format!("Error unblacklisting: {}", e))
}

fn _unblacklist(address: String) -> Result<()> {
    validate_caller()?;
    whitelist::unblacklist(&address);
    Ok(())
}

#[update]
fn is_whitelisted(address: String) -> Result<bool, String> {
    _is_whitelisted(address).map_err(|e| format!("Error checking whitelist: {}", e))
}

fn _is_whitelisted(address: String) -> Result<bool> {
    validate_caller()?;
    Ok(whitelist::is_whitelisted(&address))
}

#[update]
fn get_whitelist() -> Result<Whitelist, String> {
    _get_whitelist().map_err(|e| format!("Error getting whitelist: {}", e))
}

fn _get_whitelist() -> Result<Whitelist> {
    validate_caller()?;
    Ok(whitelist::get_list())
}

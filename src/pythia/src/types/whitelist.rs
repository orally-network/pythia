use ic_cdk::export::{
    candid::CandidType,
    serde::{Deserialize, Serialize},
};

use crate::{log, STATE};

use super::logger::WHITELIST;

#[derive(Clone, Debug, Default, Serialize, Deserialize, CandidType)]
pub struct WhitelistEntry {
    pub address: String,
    pub is_blacklisted: bool,
}

pub type Whitelist = Vec<WhitelistEntry>;

pub fn is_whitelisted(address: &str) -> bool {
    STATE.with(|state| {
        state
            .borrow()
            .whitelist
            .iter()
            .any(|entry| entry.address == address && !entry.is_blacklisted)
    })
}

pub fn add(address: &str) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let entry = WhitelistEntry {
            address: address.to_string(),
            is_blacklisted: false,
        };
        state.whitelist.push(entry);

        log!("[{WHITELIST}] Address added: {}", address);
    })
}

pub fn remove(address: &str) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.whitelist.retain(|entry| entry.address != address);
        log!("[{WHITELIST}] Address removed: {}", address);
    })
}

pub fn blacklist(address: &str) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.whitelist.iter_mut().for_each(|entry| {
            if entry.address == address {
                entry.is_blacklisted = true;
            }
        });
        log!("[{WHITELIST}] Address balcklisted: {}", address);
    })
}

pub fn unblacklist(address: &str) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.whitelist.iter_mut().for_each(|entry| {
            if entry.address == address {
                entry.is_blacklisted = false;
            }
        });
        log!("[{WHITELIST}] Address unbalcklisted: {}", address);
    })
}

pub fn get_list() -> Whitelist {
    STATE.with(|state| state.borrow().whitelist.clone())
}

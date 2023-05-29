mod methods;
mod migrations;
mod types;
mod utils;

use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;

use types::{
    chains::Chain,
    errors::PythiaError,
    subs::{CandidSub, Sub},
    U256,
};

use ic_cdk::{
    api::management_canister::http_request::{HttpResponse, TransformArgs},
    export::{candid::Nat, Principal},
    spawn,
};
use ic_cdk_macros::init;
use ic_cdk_timers::set_timer;
use ic_web3::types::H160;

thread_local! {
    pub static CONTROLLERS: RefCell<Vec<Principal>> = RefCell::default();
    pub static CHAINS: RefCell<HashMap<U256, Chain>> = RefCell::default();
    pub static TX_FEE: RefCell<U256> = RefCell::default();
    pub static KEY_NAME: RefCell<String> = RefCell::default();
    pub static SIWE_CANISTER: RefCell<Option<Principal>> = RefCell::default();
    pub static SYBIL_CANISTER: RefCell<Option<Principal>> = RefCell::default();
    pub static SUBS_LIMIT_WALLET: RefCell<u64> = RefCell::default();
    pub static SUBS_LIMIT_TOTAL: RefCell<u64> = RefCell::default();
    pub static SUBS: RefCell<Vec<Sub>> = RefCell::default();
    pub static EXEC_ADDRS: RefCell<HashMap<H160, H160>> = RefCell::default();
}

#[ic_cdk_macros::query]
fn transform(response: TransformArgs) -> HttpResponse {
    response.response
}

#[init]
fn init(tx_fee: Nat, key_name: String, siwe_canister: Principal, sybil_canister: Principal) {
    set_timer(Duration::from_secs(5), || {
        spawn(async {
            methods::controllers::update_controllers().await;
        })
    });

    TX_FEE.with(|tx_fee_state| {
        *tx_fee_state.borrow_mut() = U256::from(tx_fee);
    });

    KEY_NAME.with(|key_name_state| {
        *key_name_state.borrow_mut() = key_name;
    });

    SIWE_CANISTER.with(|siwe_canister_state| {
        *siwe_canister_state.borrow_mut() = Some(siwe_canister);
    });

    SYBIL_CANISTER.with(|sybil_canister_state| {
        *sybil_canister_state.borrow_mut() = Some(sybil_canister);
    });

    SUBS_LIMIT_WALLET.with(|subs_limit_wallet| {
        *subs_limit_wallet.borrow_mut() = 5;
    });

    SUBS_LIMIT_TOTAL.with(|subs_limit_total| {
        *subs_limit_total.borrow_mut() = 100;
    });
}

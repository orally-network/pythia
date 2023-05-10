mod methods;
mod types;
mod utils;

use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;

use types::{chains::Chain, errors::PythiaError, subs::Sub, users::User, U256};

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
    pub static USERS: RefCell<HashMap<H160, User>> = RefCell::default();
    pub static SIWE_CANISTER: RefCell<Option<Principal>> = RefCell::default();
}

#[ic_cdk_macros::query]
fn transform(response: TransformArgs) -> HttpResponse {
    response.response
}

#[init]
fn init(tx_fee: Nat, key_name: String, siwe_canister: Principal) {
    set_timer(Duration::ZERO, || {
        spawn(async {
            methods::controllers::get_controllers().await;
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
}

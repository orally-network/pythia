#[cfg(test)]
mod tests;

mod jobs;
mod methods;
mod migrations;
mod types;
mod utils;

use std::cell::RefCell;

use types::{chains::Chain, errors::PythiaError, state::State, timer::Timer};

use ic_cdk::{
    api::management_canister::http_request::{HttpResponse, TransformArgs},
    export::{candid::Nat, Principal},
};
use ic_cdk_macros::{init, query};

thread_local! {
    pub static STATE: RefCell<State> = RefCell::default();
}

#[query]
fn transform(response: TransformArgs) -> HttpResponse {
    response.response
}

#[init]
fn init(tx_fee: Nat, key_name: String, siwe_canister: Principal, sybil_canister: Principal) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.tx_fee = tx_fee;
        state.key_name = key_name;
        state.siwe_canister = Some(siwe_canister);
        state.sybil_canister = Some(sybil_canister);
        state.subs_limit_wallet = 5.into();
        state.subs_limit_total = 100.into();
        state.timer_frequency = (5 * 60).into();
        state.timer = Some(Timer::default());
    })
}

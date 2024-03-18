mod jobs;
mod methods;
mod migrations;
mod types;
mod utils;

use std::cell::RefCell;

use candid::{Nat, Principal};
use ic_web3_rs::transforms::processors;
use ic_web3_rs::transforms::transform::TransformProcessor;
use types::{chains::Chain, errors::PythiaError, state::State, timer::Timer};

use ic_cdk::{
    api::management_canister::http_request::{HttpResponse, TransformArgs},
    init, query,
};
use utils::canister::set_custom_panic_hook;

thread_local! {
    pub static STATE: RefCell<State> = RefCell::default();
}

#[query]
fn transform(response: TransformArgs) -> HttpResponse {
    HttpResponse {
        status: response.response.status,
        body: response.response.body,
        headers: Vec::new(),
    }
}

#[query]
fn transform_tx_with_logs(args: TransformArgs) -> HttpResponse {
    utils::processors::raw_tx_execution_transform_processor().transform(args)
}

#[query]
fn transform_tx(args: TransformArgs) -> HttpResponse {
    processors::send_transaction_processor().transform(args)
}

#[init]
fn init(tx_fee: Nat, key_name: String, sybil_canister: Principal) {
    set_custom_panic_hook();

    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.tx_fee = tx_fee;
        state.key_name = key_name;
        state.sybil_canister = Some(sybil_canister);
        state.subs_limit_wallet = 5.into();
        state.subs_limit_total = 100.into();
        state.timer_frequency = (5 * 60).into();
        state.timer = Some(Timer::default());
    })
}

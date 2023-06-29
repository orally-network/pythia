mod jobs;
mod methods;
mod migrations;
mod types;
mod utils;

use std::cell::RefCell;

use ic_web3_rs::transforms::{processors, transform::TransformProcessor};
use types::{chains::Chain, errors::PythiaError, state::State, timer::Timer};

use ic_cdk::{
    api::management_canister::http_request::{HttpResponse, TransformArgs},
    export::{candid::Nat, Principal},
    init, query,
};

thread_local! {
    pub static STATE: RefCell<State> = RefCell::default();
}

#[query]
fn transform(response: TransformArgs) -> HttpResponse {
    response.response
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

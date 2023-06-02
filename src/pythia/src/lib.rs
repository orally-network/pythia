mod methods;
mod migrations;
mod types;
mod utils;

use std::cell::RefCell;
use std::time::Duration;

use types::{
    chains::Chain,
    errors::PythiaError,
    subs::{CandidSub, Sub},
    state::State,
};

use ic_cdk::{
    api::management_canister::http_request::{HttpResponse, TransformArgs},
    export::{candid::Nat, Principal},
    spawn,
};
use ic_cdk_macros::init;
use ic_cdk_timers::set_timer;

thread_local! {
    pub static STATE: RefCell<State> = RefCell::default();
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

    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.tx_fee = tx_fee;
        state.key_name = key_name;
        state.siwe_canister = Some(siwe_canister);
        state.sybil_canister = Some(sybil_canister);
        state.subs_limit_wallet = 5;
        state.subs_limit_total = 100;
    })
}

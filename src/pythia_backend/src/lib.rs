mod utils;
mod types;
mod chains;
mod users;
mod subs;

use std::cell::RefCell;
use std::time::Duration;
use std::collections::HashMap;

use types::{
    U256,
    errors::PythiaError,
    chains::Chain,
    users::User,
    subs::Sub,
};

use ic_cdk_macros::init;
use ic_cdk::{
    spawn,
    export::{
        Principal,
        candid::Nat,
    }
};
use ic_cdk_timers::set_timer;

thread_local!{
    pub static CONTROLLERS: RefCell<Vec<Principal>> = RefCell::default();
    pub static CHAINS: RefCell<HashMap<U256, Chain>> = RefCell::default();
    pub static TX_FEE: RefCell<U256> = RefCell::default();
    pub static KEY_NAME: RefCell<String> = RefCell::default();
    pub static USERS: RefCell<HashMap<Principal, User>> = RefCell::default();
    pub static MIN_BALANCE: RefCell<U256> = RefCell::default();
}

#[init]
fn init(tx_fee: Nat, key_name: String, min_balance: Nat) {
    set_timer(Duration::ZERO, || spawn(async {
        utils::get_controllers().await;
    }));

    TX_FEE.with(|tx_fee_state| {
        *tx_fee_state.borrow_mut() = U256::from(&tx_fee);
    });

    KEY_NAME.with(|key_name_state| {
        *key_name_state.borrow_mut() = key_name;
    });

    MIN_BALANCE.with(|min_balance_state| {
        *min_balance_state.borrow_mut() = U256::from(&min_balance);
    });
}

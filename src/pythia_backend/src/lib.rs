mod utils;
mod types;
mod chains;

use std::cell::RefCell;
use std::time::Duration;
use std::collections::HashMap;

use futures::executor;
use types::{U256, Chain};

use ic_cdk_macros::init;
use ic_cdk::export::Principal;
use ic_cdk_timers::set_timer;

thread_local!{
    pub static CONTROLLERS: RefCell<Vec<Principal>> = RefCell::default();
    pub static CHAINS: RefCell<HashMap<U256, Chain>> = RefCell::default();
}

#[init]
fn init() {
    set_timer(Duration::from_nanos(1), || {
        executor::block_on(utils::get_controllers());
    });
}

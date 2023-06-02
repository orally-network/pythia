use std::str::FromStr;
use std::time::Duration;

use ic_cdk::storage;
use ic_cdk_macros::{post_upgrade, pre_upgrade};
use ic_cdk_timers::set_timer_interval;
use ic_utils::{logger, monitor};
use ic_web3::types::H160;

use crate::{
    utils::publish::publish, STATE, State,
};

#[pre_upgrade]
fn pre_upgrade() {
    let state = STATE.with(|state| state.take());

    let log_data = logger::pre_upgrade_stable_data();
    let monitor_data = monitor::pre_upgrade_stable_data();

    storage::stable_save((
        state,
        log_data,
        monitor_data,
    ))
    .expect("should be valid canister data for pre upgrade");
}

#[post_upgrade]
fn post_upgrade() {
    #[allow(clippy::type_complexity)]
    let (
        state,
        log_data,
        monitor_data,
    ): (
        State,
        logger::PostUpgradeStableData,
        monitor::PostUpgradeStableData,
    ) = storage::stable_restore().expect("should be valid canister data for post upgrade");

    logger::post_upgrade_stable_data(log_data);
    monitor::post_upgrade_stable_data(monitor_data);

    STATE.with(|s| s.replace(state));

    STATE.with(|state| {
        let mut state = state.borrow_mut();
        for sub in state.subs.iter_mut() {
            if sub.is_active {
                let id = sub.id;
                let pub_key = H160::from_str(&sub.owner)
                    .expect("should be valid subscription owner address");

                let timer_id = set_timer_interval(Duration::from_secs(sub.frequency), move || {
                    publish(id, pub_key);
                });

                sub.timer_id = serde_json::to_string(&timer_id).expect("should be valid timer id");
            }
        }
    });
}

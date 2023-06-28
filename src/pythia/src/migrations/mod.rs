use std::time::Duration;

use ic_cdk::{post_upgrade, pre_upgrade, storage};
use ic_cdk_timers::set_timer;
use ic_utils::{logger, monitor};

use crate::{jobs::publisher, types::timer::Timer, State, STATE};

#[pre_upgrade]
fn pre_upgrade() {
    let state = STATE.with(|state| state.take());

    let log_data = logger::pre_upgrade_stable_data();
    let monitor_data = monitor::pre_upgrade_stable_data();

    storage::stable_save((state, log_data, monitor_data))
        .expect("should be valid canister data for pre upgrade");
}

#[post_upgrade]
fn post_upgrade() {
    #[allow(clippy::type_complexity)]
    let (mut state, log_data, monitor_data): (
        State,
        logger::PostUpgradeStableData,
        monitor::PostUpgradeStableData,
    ) = storage::stable_restore().expect("should be valid canister data for post upgrade");

    logger::post_upgrade_stable_data(log_data);
    monitor::post_upgrade_stable_data(monitor_data);

    let timer_id = set_timer(Duration::from_secs(10), publisher::execute);
    let timer = Timer {
        id: serde_json::to_string(&timer_id).unwrap(),
        is_active: true,
    };

    state.timer = Some(timer);

    STATE.with(|s| s.replace(state));
}

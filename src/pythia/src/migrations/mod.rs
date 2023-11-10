use std::time::Duration;

use ic_cdk::{post_upgrade, pre_upgrade, storage};
use ic_cdk_timers::set_timer;
use ic_utils::{logger, monitor};

use crate::{
    jobs::publisher,
    types::{methods::ExecutionCondition, timer::Timer},
    utils::nat,
    State, STATE,
};

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

    state
        .subscriptions
        .0
        .iter_mut()
        .for_each(|(_, subscriptions)| {
            for subscription in subscriptions {
                if subscription.method.exec_condition.is_none() {
                    subscription.method.exec_condition = Some(ExecutionCondition::Frequency(
                        nat::to_u64(&subscription.old_frequency),
                    ));
                }
            }
        });

    logger::post_upgrade_stable_data(log_data);
    monitor::post_upgrade_stable_data(monitor_data);

    let timer_id = set_timer(Duration::from_secs(10), publisher::execute);
    let timer = Timer {
        id: serde_json::to_string(&timer_id).expect("should be valid timer id"),
        is_active: true,
    };

    state.timer = Some(timer);

    state.chains.0.iter_mut().for_each(|(_, chain)| {
        if chain.multicall_contract.is_none() {
            chain.multicall_contract = Some(OLD_MULTICALL_CONTRACT_ADDRESS.to_string());
        }
    });

    STATE.with(|s| s.replace(state));

    _ = std::panic::take_hook(); // clear custom panic hook and set default
    let old_handler = std::panic::take_hook(); // take default panic hook

    // set custom panic hook
    std::panic::set_hook(Box::new(move |info| {
        log!("PANIC OCCURRED: {:#?}", info);
        old_handler(info);
    }));

    log!("post upgrade finished");
}

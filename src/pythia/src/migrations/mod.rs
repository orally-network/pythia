use std::collections::HashMap;
use std::time::Duration;

use ic_cdk::{export::Principal, storage};
use ic_cdk_macros::{post_upgrade, pre_upgrade};
use ic_cdk_timers::set_timer_interval;
use ic_utils::{logger, monitor};
use ic_web3::types::H160;

use crate::{
    types::subs::Sub, utils::publish::publish, Chain, CHAINS, CONTROLLERS, EXEC_ADDRS, KEY_NAME,
    SIWE_CANISTER, SUBS, SUBS_LIMIT_TOTAL, SUBS_LIMIT_WALLET, SYBIL_CANISTER, TX_FEE, U256,
};

#[pre_upgrade]
fn pre_upgrade() {
    let controllers = CONTROLLERS.with(|controllers| controllers.take());
    let chains = serde_json::to_string(&CHAINS.with(|chains| chains.take()))
        .expect("should be valid chains data");
    let tx_fee = serde_json::to_string(&TX_FEE.with(|tx_fee| tx_fee.take()))
        .expect("should be valid tx fee");
    let key_name = KEY_NAME.with(|key_name| key_name.take());
    let siwe_canister = SIWE_CANISTER.with(|siwe_canister| siwe_canister.take());
    let sybil_canister = SYBIL_CANISTER.with(|sybil_canister| sybil_canister.take());
    let subs_limit_wallet = SUBS_LIMIT_WALLET.with(|subs_limit_wallet| subs_limit_wallet.take());
    let subs_limit_total = SUBS_LIMIT_TOTAL.with(|subs_limit_wallet| subs_limit_wallet.take());
    let subs = serde_json::to_string(&SUBS.with(|subs| subs.take())).expect("should be valid subs");
    let exec_addrs = serde_json::to_string(&EXEC_ADDRS.with(|subs| subs.take()))
        .expect("should be valid exec addrs");

    let log_data = logger::pre_upgrade_stable_data();
    let monitor_data = monitor::pre_upgrade_stable_data();

    storage::stable_save((
        controllers,
        chains,
        tx_fee,
        key_name,
        siwe_canister,
        sybil_canister,
        subs_limit_wallet,
        subs_limit_total,
        subs,
        exec_addrs,
        log_data,
        monitor_data,
    ))
    .expect("should be valid canister data");
}

#[post_upgrade]
fn post_upgrade() {
    #[allow(clippy::type_complexity)]
    let (
        controllers,
        chains,
        tx_fee,
        key_name,
        siwe_canister,
        sybil_canister,
        subs_limit_wallet,
        subs_limit_total,
        subs,
        exec_addrs,
        log_data,
        monitor_data,
    ): (
        Vec<Principal>,
        String,
        String,
        String,
        Option<Principal>,
        Option<Principal>,
        u64,
        u64,
        String,
        String,
        logger::PostUpgradeStableData,
        monitor::PostUpgradeStableData,
    ) = storage::stable_restore().expect("should be valid canister data");

    let chains: HashMap<U256, Chain> =
        serde_json::from_str(&chains).expect("should be valid chains data");

    let subs: Vec<Sub> = serde_json::from_str(&subs).expect("should be valid subs data");

    let exec_addrs: HashMap<H160, H160> =
        serde_json::from_str(&exec_addrs).expect("should be valid exec addrs data");

    let tx_fee: U256 = serde_json::from_str(&tx_fee).expect("should be valid tx fee");

    logger::post_upgrade_stable_data(log_data);
    monitor::post_upgrade_stable_data(monitor_data);

    CONTROLLERS.with(|c| c.replace(controllers));
    CHAINS.with(|c| c.replace(chains));
    TX_FEE.with(|t| t.replace(tx_fee));
    KEY_NAME.with(|k| k.replace(key_name));
    SIWE_CANISTER.with(|s| s.replace(siwe_canister));
    SYBIL_CANISTER.with(|s| s.replace(sybil_canister));
    SUBS_LIMIT_WALLET.with(|s| s.replace(subs_limit_wallet));
    SUBS_LIMIT_TOTAL.with(|s| s.replace(subs_limit_total));
    SUBS.with(|s| s.replace(subs));
    EXEC_ADDRS.with(|s| s.replace(exec_addrs));

    SUBS.with(|subs| {
        let mut subs = subs.borrow_mut();
        for sub in subs.iter_mut() {
            if sub.is_active {
                let id = sub.id;
                let pub_key = sub.owner;

                let timer_id = set_timer_interval(Duration::from_secs(sub.frequency), move || {
                    publish(id, pub_key);
                });

                sub.timer_id = serde_json::to_string(&timer_id).expect("should be valid timer id");
            }
        }
    });
}

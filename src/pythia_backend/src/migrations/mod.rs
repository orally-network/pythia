use std::collections::HashMap;

use ic_cdk::{export::Principal, storage};
use ic_cdk_macros::{post_upgrade, pre_upgrade};
use ic_utils::logger;
use ic_web3::types::H160;

use crate::{Chain, User, CHAINS, CONTROLLERS, KEY_NAME, SIWE_CANISTER, TX_FEE, U256, USERS};

#[pre_upgrade]
fn pre_upgrade() {
    let controllers = CONTROLLERS.with(|controllers| controllers.take());
    let chains = serde_json::to_string(&CHAINS.with(|chains| chains.take()))
        .expect("should be valid chains data");
    let users = serde_json::to_string(&USERS.with(|users| users.take()))
        .expect("should be valid users data");
    let tx_fee = serde_json::to_string(&TX_FEE.with(|tx_fee| tx_fee.take()))
        .expect("should be valid tx fee");
    let key_name = KEY_NAME.with(|key_name| key_name.take());
    let siwe_canister = SIWE_CANISTER.with(|siwe_canister| siwe_canister.take());

    let log_data = logger::pre_upgrade_stable_data();

    storage::stable_save((
        controllers,
        chains,
        users,
        tx_fee,
        key_name,
        siwe_canister,
        log_data,
    ))
    .expect("should be valid canister data");
}

#[post_upgrade]
fn post_upgrade() {
    let (controllers, chains, users, tx_fee, key_name, siwe_canister, log_data): (
        Vec<Principal>,
        String,
        String,
        String,
        String,
        Option<Principal>,
        logger::PostUpgradeStableData,
    ) = storage::stable_restore().expect("should be valid canister data");

    let chains: HashMap<U256, Chain> =
        serde_json::from_str(&chains).expect("should be valid chains data");

    let users: HashMap<H160, User> =
        serde_json::from_str(&users).expect("should be valid users data");

    let tx_fee: U256 = serde_json::from_str(&tx_fee).expect("should be valid tx fee");

    logger::post_upgrade_stable_data(log_data);

    CONTROLLERS.with(|c| c.replace(controllers));
    CHAINS.with(|c| c.replace(chains));
    USERS.with(|u| u.replace(users));
    TX_FEE.with(|t| t.replace(tx_fee));
    KEY_NAME.with(|k| k.replace(key_name));
    SIWE_CANISTER.with(|s| s.replace(siwe_canister));
}

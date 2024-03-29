use std::{sync::Arc, time::Duration};

use anyhow::Result;
use candid::Principal;
use ic_cdk::call;

use crate::log;

pub mod abi;
pub mod address;
pub mod canister;
pub mod macros;
pub mod metrics;
pub mod multicall;
pub mod nat;
pub mod processors;
pub mod signature;
pub mod siwe;
pub mod sybil;
pub mod time;
pub mod validator;
pub mod web3;

pub async fn sleep(dur: Duration) {
    let notify = Arc::new(tokio::sync::Notify::new());
    let notifyer = notify.clone();

    log!("Sleeping for {}ms", dur.as_millis());
    ic_cdk_timers::set_timer(dur, move || {
        notifyer.notify_one();
    });

    notify.notified().await;
    log!("Sleeping finished");
}

pub async fn rand() -> Result<Vec<u8>> {
    let (bytes,): (Vec<u8>,) = call(Principal::management_canister(), "raw_rand", ())
        .await
        .map_err(|err| anyhow::anyhow!(format!("{:?}", err)))?;

    Ok(bytes)
}

use ic_cdk::api::{management_canister::main::raw_rand, time};

#[inline]
pub fn in_seconds() -> u64 {
    time() / 1_000_000_000
}

#[allow(dead_code)]
pub async fn wait(delay: u64) {
    let end = in_seconds() + delay;
    while in_seconds() < end {
        let _ = raw_rand().await;
    }
}

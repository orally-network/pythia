use ic_cdk::api::time;

#[inline]
pub fn in_seconds() -> u64 {
    time() / 1_000_000_000
}

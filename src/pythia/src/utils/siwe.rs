use anyhow::{anyhow, Result};

use super::address;
use crate::clone_with_state;

pub async fn recover(msg: &str, sig: &str) -> Result<String> {
    let siwe_canister = clone_with_state!(siwe_canister).expect("canister should be initialized");

    let (signer,): (String,) = ic_cdk::call(
        siwe_canister,
        "get_signer",
        (msg.to_string(), sig.to_string()),
    )
    .await
    .map_err(|(code, msg)| anyhow!("{:?}: {}", code, msg))?;

    address::normalize(&signer)
}

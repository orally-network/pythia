use std::str::FromStr;

use anyhow::Result;
use siwe::{Message, VerificationOpts};
use time::OffsetDateTime;
// use time::OffsetDateTime;

use super::address;
use ic_cdk::api::time;
use std::time::Duration;

pub async fn siwe_recover(msg: &str, sig: &str) -> Result<String> {
    let msg = Message::from_str(&msg).expect("must be valid message");

    let sig = hex::decode(sig).expect("must be valid hex");

    let timestamp = OffsetDateTime::from_unix_timestamp(
        (Duration::from_nanos(time()).as_secs() / 1_000_000_000) as i64,
    )
    .expect("must be valid timestamp");

    let opts = VerificationOpts {
        timestamp: Some(timestamp),
        ..Default::default()
    };

    msg.verify(&sig, &opts)
        .await
        .expect("must be valid signature");

    let signer = hex::encode(msg.address);

    address::normalize(&signer)
}

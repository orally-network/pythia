use std::str::FromStr;

#[allow(unused_imports)]
use siwe::{Message, VerificationOpts};
#[allow(unused_imports)]
use time::OffsetDateTime;

use ic_cdk_macros::query;

#[query]
pub async fn get_signer(msg: String, sig: String) -> String {
    let msg = Message::from_str(&msg).expect("must be valid message");

    let sig = hex::decode(sig).expect("must be valid hex");

    let timestamp =
        OffsetDateTime::from_unix_timestamp((ic_cdk::api::time() / 1_000_000_000) as i64)
            .expect("must be valid timestamp");

    let opts = VerificationOpts {
        timestamp: Some(timestamp),
        ..Default::default()
    };

    msg.verify(&sig, &opts)
        .await
        .expect("must be valid signature");

    hex::encode(msg.address)
}

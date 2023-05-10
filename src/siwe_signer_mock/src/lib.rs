use std::str::FromStr;

#[allow(unused_imports)]
use siwe::{Message, VerificationOpts};
#[allow(unused_imports)]
use time::OffsetDateTime;

use ic_cdk_macros::query;

#[query]
#[allow(unused_variables)]
pub async fn get_signer(msg: String, sig: String) -> String {
    let msg = Message::from_str(&msg).expect("must be valid message");

    hex::encode(msg.address)
}

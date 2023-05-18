use anyhow::Result;

use ic_cdk_macros::update;
use ic_utils::logger::log_message;

use crate::{utils::rec_eth_addr, User, USERS};

#[update]
pub async fn add_user(msg: String, sig: String) -> Result<String, String> {
    _add_user(msg, sig).await.map_err(|e| e.to_string())
}

async fn _add_user(msg: String, sig: String) -> Result<String> {
    let pub_key = rec_eth_addr(&msg, &sig).await?;
    let user = User::new(pub_key).await?;
    let exec_addr = hex::encode(user.exec_addr.as_bytes());

    USERS.with(|users_state| {
        users_state.borrow_mut().insert(pub_key, user);
    });

    log_message(format!(
        "[USER: {pub_key}] creation, exec_addr: {exec_addr}"
    ));

    Ok(exec_addr)
}

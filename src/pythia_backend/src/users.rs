use anyhow::Result;

use ic_cdk_macros::update;

use crate::{
    User,
    USERS,
    utils::rec_eth_addr
};

#[update]
pub async fn add_user(msg: String, sig: String) -> Result<String, String> {
    let pub_key = rec_eth_addr(&msg, &sig)
        .await
        .map_err(|e| format!("failed to recover a public key: {}", e))?;
    
    let user = User::new(pub_key)
        .await
        .map_err(|e| format!("{}", e))?;

    let exec_addr = user.exec_addr.to_string();

    USERS.with(|users_state| {
        users_state.borrow_mut().insert(ic_cdk::caller(), user);
    });

    Ok(exec_addr)
}

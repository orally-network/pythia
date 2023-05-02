use anyhow::Result;

use ic_cdk_macros::update;
use ic_cdk::export::candid::Nat;

use crate::{
    CHAINS,
    USERS,
    U256,
    PythiaError,
    Sub,
    utils::check_balance,
};

#[update]
pub async fn subscribe(
    chain_id: Nat,
    contract_addr: String,
    method_abi: Vec<u8>,
    frequency: u64,
) -> Result<(), String> {
    let chain_id = U256::from(&chain_id);
    let rpc = CHAINS.with(|chains| {
        Ok(chains
            .borrow()
            .get(&chain_id)
            .ok_or(PythiaError::ChainDoesNotExist)?
            .rpc.clone())
    })
    .map_err(|e: PythiaError| format!("{}", e))?;

    let user = USERS.with(|users| {
        Ok(users.
            borrow()
            .get(&ic_cdk::caller())
            .ok_or(PythiaError::UserNotFound)?
            .clone())
    })
    .map_err(|e: PythiaError| format!("{}", e))?;

    check_balance(&user.exec_addr, &rpc)
        .await
        .map_err(|e| format!("{}", e))?;

    let sub = Sub::new(
        &chain_id,
        &contract_addr,
        &method_abi,
        &frequency,
    )
    .map_err(|e| format!("{}", e))?;

    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&ic_cdk::caller()).unwrap();
        user.subs.push(sub.clone());
    });

    Ok(())
}
pub mod chains;
pub mod controllers;
pub mod subs;
pub mod users;

use anyhow::Result;
use ic_cdk::{query, update};
use ic_utils::{
    api_type::{GetInformationRequest, GetInformationResponse, UpdateInformationRequest},
    get_information, update_information,
};
use ic_web3::{ic::get_eth_addr, types::H160};

use crate::{types::errors::PythiaError, utils::rec_eth_addr, EXEC_ADDRS, KEY_NAME};

#[query(name = "getCanistergeekInformation")]
pub async fn get_canistergeek_information(
    request: GetInformationRequest,
) -> GetInformationResponse<'static> {
    get_information(request)
}

#[update(name = "updateCanistergeekInformation")]
pub async fn update_canistergeek_information(request: UpdateInformationRequest) {
    update_information(request);
}

#[update]
pub async fn get_exec_addr(msg: String, sig: String) -> Result<String, String> {
    _get_exec_addr(msg, sig).await.map_err(|e| format!("{e:?}"))
}

async fn _get_exec_addr(msg: String, sig: String) -> Result<String> {
    let pub_key = rec_eth_addr(&msg, &sig).await?;

    let exec_addr = get_exec_addr_from_pub(&pub_key).await?;

    Ok(hex::encode(exec_addr.as_bytes()))
}

pub async fn get_exec_addr_from_pub(pub_key: &H160) -> Result<H160> {
    let exec_addr = EXEC_ADDRS.with(|exec_addr| exec_addr.borrow().get(pub_key).copied());

    if let Some(exec_addr) = exec_addr {
        return Ok(exec_addr);
    }

    let derivation_path = vec![pub_key.as_bytes().to_vec()];
    let key_name = KEY_NAME.with(|key_name| key_name.borrow().clone());
    let exec_addr = get_eth_addr(None, Some(derivation_path), key_name)
        .await
        .map_err(PythiaError::FailedToGetEthAddress)?;

    EXEC_ADDRS.with(|exec_addrs| exec_addrs.borrow_mut().insert(*pub_key, exec_addr));

    Ok(exec_addr)
}

pub mod chains;
pub mod controllers;
pub mod subs;
pub mod users;

use std::str::FromStr;

use anyhow::Result;
use ic_cdk::{query, update};
use ic_utils::{
    api_type::{GetInformationRequest, GetInformationResponse, UpdateInformationRequest},
    get_information, update_information,
};
use ic_web3::{ic::get_eth_addr, types::H160};

use crate::{types::errors::PythiaError, utils::rec_eth_addr, STATE};

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
    let exec_addr = STATE.with(|state| {
        state.borrow().exec_addrs.get(&hex::encode(pub_key.as_bytes())).cloned()
    });

    if let Some(exec_addr) = exec_addr {
        return Ok(H160::from_str(&exec_addr)
            .expect("should be valid execution address"));
    }

    let derivation_path = vec![pub_key.as_bytes().to_vec()];
    let key_name = STATE.with(|state| state.borrow().key_name.clone());
    let exec_addr = get_eth_addr(None, Some(derivation_path), key_name)
        .await
        .map_err(PythiaError::FailedToGetEthAddress)?;

    STATE.with(|state| {
        state.borrow_mut().exec_addrs.insert(hex::encode(pub_key.as_bytes()), hex::encode(exec_addr.as_bytes()))
    });

    Ok(exec_addr)
}

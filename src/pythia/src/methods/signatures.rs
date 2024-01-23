use anyhow::Result;
use ic_cdk::{
    api::management_canister::ecdsa::{EcdsaCurve, EcdsaKeyId, SignWithEcdsaArgument},
    update,
};
use ic_web3_rs::signing::keccak256;

use crate::{
    clone_with_state,
    types::errors::PythiaError,
    utils::{
        address, canister,
        signature::{get_eth_v, sign},
        validator,
    },
};

#[update]
async fn sign_message(message: String) -> Result<String, String> {
    _sign_message(message)
        .await
        .map_err(|e| format!("Failed to sign message: {}", e))
}

#[inline]
async fn _sign_message(message: String) -> Result<String> {
    validator::caller()?;
    let sign_data = keccak256(message.as_bytes()).to_vec();

    let key_name = clone_with_state!(key_name);
    let call_args = SignWithEcdsaArgument {
        message_hash: sign_data.clone(),
        derivation_path: vec![vec![]],
        key_id: EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: key_name,
        },
    };

    let mut signature = sign(call_args)
        .await
        .map_err(|(_, msg)| PythiaError::SignError(msg))?
        .0
        .signature;

    let pub_key = canister::pma().await?;

    signature.push(get_eth_v(
        &signature,
        &sign_data,
        &address::to_h160(&pub_key)?,
    )?);

    Ok(hex::encode(signature))
}

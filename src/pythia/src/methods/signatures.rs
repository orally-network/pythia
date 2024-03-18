use std::str::FromStr;

use anyhow::Result;
use candid::Nat;
use ic_cdk::{
    api::management_canister::ecdsa::{EcdsaCurve, EcdsaKeyId, SignWithEcdsaArgument},
    query, update,
};
use ic_web3_rs::signing::keccak256;
use siwe::Message;
use time::OffsetDateTime;

use crate::{
    clone_with_state,
    types::errors::PythiaError,
    utils::{
        address, canister,
        signature::{get_eth_v, sign},
        siwe::siwe_recover,
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

    let eip191_message = format!(
        "\x19Ethereum Signed Message:\n{}{}",
        message.as_bytes().len(),
        message
    );

    let sign_data = keccak256(eip191_message.as_bytes()).to_vec();

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

#[query]
async fn verify_signed_message(message: String, signature: String) -> Result<(), String> {
    if signature.len() != 130 {
        return Err("Invalid signature length".to_string());
    }

    let eip191_message = format!(
        "\x19Ethereum Signed Message:\n{}{}",
        message.as_bytes().len(),
        message
    );

    let data = keccak256(eip191_message.as_bytes()).to_vec();

    let mut signature = hex::decode(signature)
        .map_err(|err| format!("Failed to decode hex signature: {:?}", err))?;

    let rec_id = signature.pop().unwrap() % 27;

    let recovered_address =
        address::normalize(&ic_web3_rs::ic::recover_address(data, signature, rec_id))
            .map_err(|err| format!("Failed to recover address: {:?}", err))?;

    let canister_pma = canister::pma()
        .await
        .map_err(|err| format!("Failed to get pma: {:?}", err))?;

    if canister_pma == recovered_address {
        Ok(())
    } else {
        Err("Invalid signature".to_string())
    }
}

#[derive(candid::CandidType, serde::Deserialize, serde::Serialize)]
pub struct SIWESignedMessage {
    pub signature: String,
    pub message: String,
}

#[update]
async fn siwe_sign_message(message: String, chain_id: Nat) -> Result<SIWESignedMessage, String> {
    _siwe_sign_message(message, chain_id)
        .await
        .map_err(|e| format!("Failed to sign message: {}", e))
}

#[inline]
async fn _siwe_sign_message(message: String, chain_id: Nat) -> Result<SIWESignedMessage> {
    validator::caller()?;

    let rand_bytes = crate::utils::rand().await?;
    let nonce = hex::encode(&rand_bytes[..17]);

    let address = address::eip55(canister::pma().await?)?;
    let timestamp = OffsetDateTime::from_unix_timestamp(
        (std::time::Duration::from_nanos(ic_cdk::api::time()).as_secs()) as i64,
    )
    .expect("must be valid timestamp");

    let iso8601 = timestamp
        .format(&time::format_description::well_known::Iso8601::DEFAULT)
        .unwrap();

    let siwe_msg = format!(
        r#"Pythia wants you to sign in with your Ethereum account:
{}

{}

URI: http://localhost:8080
Version: 1
Chain ID: {}
Nonce: {}
Issued At: {}"#,
        address, message, chain_id, nonce, iso8601
    );

    let msg = Message::from_str(&siwe_msg).unwrap();
    let sign_data = msg.eip191_hash().unwrap().to_vec();

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

    let encoded = hex::encode(signature);

    siwe_recover(&siwe_msg, &encoded).await?;

    Ok(SIWESignedMessage {
        signature: encoded,
        message: siwe_msg,
    })
}

#[query]
async fn siwe_verify_signed_message(message: String, signature: String) -> Result<(), String> {
    siwe_recover(&message, &signature)
        .await
        .map_err(|err| format!("Cannot verify signature: {:?}", err))?;

    Ok(())
}

pub mod publish;

use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use url::Url;

use ic_web3::{
    ethabi::{ParamType, Token},
    transports::ICHttp,
    types::H160,
    Web3,
};

use crate::{types::errors::PythiaError, Chain, User, CONTROLLERS, U256, SIWE_CANISTER};

pub fn validate_caller() -> Result<()> {
    let controllers = CONTROLLERS.with(|controllers| controllers.borrow().clone());

    if controllers.contains(&ic_cdk::caller()) {
        return Ok(());
    }

    Err(anyhow!(PythiaError::NotAController))
}

pub async fn rec_eth_addr(msg: &str, sig: &str) -> Result<H160> {
    let siwe_canister = SIWE_CANISTER.with(|siwe_canister| siwe_canister.borrow().clone())
        .expect("canister should be initialized");

    let msg = msg.to_string();
    let sig = sig.to_string();

    let (signer, ): (String, ) = ic_cdk::call(siwe_canister, "get_signer", (msg, sig))
        .await
        .map_err(|(code, msg)| anyhow!("{:?}: {}", code, msg))?;

    Ok(H160::from_str(&signer).context("failed to parse signer address")?)
}

pub async fn check_balance(user: &User, chain: &Chain) -> Result<()> {
    let balance = get_balance(&user.exec_addr, &chain.rpc).await?;

    if balance < chain.min_balance {
        return Err(anyhow!(PythiaError::InsufficientBalance));
    }

    Ok(())
}

pub async fn get_balance(address: &H160, rpc: &Url) -> Result<U256> {
    let w3 =
        Web3::new(ICHttp::new(rpc.as_str(), None, None).context("failed to connect to a node")?);

    let balance = w3
        .eth()
        .balance(*address, None)
        .await
        .context("failed to get balance")?;

    Ok(U256(balance))
}

pub fn add_brackets(data: &str) -> String {
    format!("[{}]", data)
}

pub fn cast_to_param_type(value: u64, kind: &ParamType) -> Option<Token> {
    match kind {
        ParamType::Bytes => Some(Token::Bytes(value.to_le_bytes().to_vec())),
        ParamType::FixedBytes(_) => Some(Token::FixedBytes(value.to_le_bytes().to_vec())),
        ParamType::Uint(_) => Some(Token::Uint(value.into())),
        ParamType::Int(_) => Some(Token::Int(value.into())),
        ParamType::String => Some(Token::String(value.to_string())),
        _ => None,
    }
}

pub mod publish;

use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use url::Url;

use ic_web3::{
    ethabi::{ParamType, Token},
    transports::ICHttp,
    types::{TransactionRequest, H160},
    Web3,
};

use crate::{types::errors::PythiaError, Chain, User, CONTROLLERS, SIWE_CANISTER, TX_FEE, U256};

pub fn validate_caller() -> Result<()> {
    let controllers = CONTROLLERS.with(|controllers| controllers.borrow().clone());

    if controllers.contains(&ic_cdk::caller()) {
        return Ok(());
    }

    Err(anyhow!(PythiaError::NotAController))
}

pub async fn rec_eth_addr(msg: &str, sig: &str) -> Result<H160> {
    let siwe_canister = SIWE_CANISTER
        .with(|siwe_canister| *siwe_canister.borrow())
        .expect("canister should be initialized");

    let msg = msg.to_string();
    let sig = sig.to_string();

    let (signer,): (String,) = ic_cdk::call(siwe_canister, "get_signer", (msg, sig))
        .await
        .map_err(|(code, msg)| anyhow!("{:?}: {}", code, msg))?;

    H160::from_str(&signer).context("failed to parse signer address")
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

pub async fn collect_fee(user: &User, chain: &Chain) -> Result<()> {
    let w3 = Web3::new(
        ICHttp::new(chain.rpc.as_str(), None, None).context("failed to connect to a node")?,
    );

    let nonce = w3
        .eth()
        .transaction_count(user.exec_addr, None)
        .await
        .context("failed to get nonce")?;

    let gas_price = w3
        .eth()
        .gas_price()
        .await
        .context("failed to get gas price")?;

    // 1.1 multiplication
    let gas_price = (gas_price / 10) * 11;

    let fee = TX_FEE.with(|fee| *fee.borrow());

    let tx_hash = w3
        .eth()
        .send_transaction(TransactionRequest {
            from: user.exec_addr,
            to: Some(chain.treasurer),
            gas: Some(21000.into()),
            gas_price: Some(gas_price),
            value: Some(fee.0),
            data: None,
            nonce: Some(nonce),
            ..Default::default()
        })
        .await
        .context("failed to send transaction")?;

    ic_cdk::println!("Fee tx_hash: {:?}", hex::encode(tx_hash.as_bytes()));

    Ok(())
}

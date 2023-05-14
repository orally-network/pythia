pub mod publish;

use std::str::FromStr;

use anyhow::{anyhow, Context, Result};

use ic_web3::{
    ethabi::Token,
    transports::ICHttp,
    types::{H160, TransactionParameters},
    ic::KeyInfo,
    Web3,
};

use crate::{types::errors::PythiaError, Chain, User, CONTROLLERS, SIWE_CANISTER, TX_FEE, U256, KEY_NAME};

const ETH_TRANSFER_GAS_LIMIT: u64 = 21000;

pub fn validate_caller() -> Result<(), PythiaError> {
    let controllers = CONTROLLERS.with(|controllers| controllers.borrow().clone());

    if controllers.contains(&ic_cdk::caller()) {
        return Ok(());
    }

    Err(PythiaError::NotAController)
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

pub async fn get_balance(address: &H160, rpc: &str) -> Result<U256> {
    let w3 =
        Web3::new(ICHttp::new(rpc, None, None).context("failed to connect to a node")?);

    let balance = w3
        .eth()
        .balance(*address, None)
        .await
        .context("failed to get balance")?;

    Ok(U256(balance))
}

#[inline]
pub fn add_brackets(data: &str) -> String {
    format!("[{}]", data)
}

pub fn cast_to_param_type(value: u64, kind: &str) -> Option<Token> {
    if kind == "bytes" { return Some(Token::Bytes(value.to_le_bytes().to_vec())) }
    if kind.contains("bytes") { return Some(Token::FixedBytes(value.to_le_bytes().to_vec()))}
    if kind.contains("uint") { return Some(Token::Uint(value.into())) }
    if kind.contains("int") { return Some(Token::Int(value.into())) }
    if kind.contains("string") { return Some(Token::String(value.to_string())) }

    None
}

pub async fn collect_fee(user: &User, chain: &Chain) -> Result<()> {
    let fee = TX_FEE.with(|fee| *fee.borrow());

    if fee == U256::from(0) {
        return Ok(());
    }

    let w3 = Web3::new(
        ICHttp::new(chain.rpc.as_str(), None, None).context("failed to connect to a node")?,
    );

    let key_info = KeyInfo {
        derivation_path: vec![user.pub_key.as_bytes().to_vec()],
        key_name: KEY_NAME.with(|key_name| key_name.borrow().clone()),
    };

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

    let tx = TransactionParameters {
        to: Some(chain.treasurer),
        nonce: Some(nonce),
        value: fee.0,
        gas_price: Some(gas_price),
        gas: ETH_TRANSFER_GAS_LIMIT.into(),
        ..Default::default()
    };

    let signed_tx = w3.accounts()
        .sign_transaction(tx, user.exec_addr.to_string(), key_info, chain.chain_id.0.as_u64())
        .await?;

    w3
        .eth()
        .send_raw_transaction(signed_tx.raw_transaction)
        .await?;

    Ok(())
}

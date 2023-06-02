pub mod publish;
pub mod sybil;
pub mod multicall;

use std::str::FromStr;

use anyhow::{anyhow, Context, Result};

use candid::Nat;
use ic_web3::{
    ethabi::Token,
    ic::KeyInfo,
    transports::ICHttp,
    types::{Bytes, TransactionParameters, H160, U256},
    Web3,
};
use num_bigint::BigUint;

use crate::{
    types::errors::PythiaError, utils::publish::wait_until_confimation, Chain, STATE,
};

const ATTEMPTS_TO_SEND_TX: u64 = 3;
const ETH_TRANSFER_GAS_LIMIT: u64 = 21000;
const ECDSA_SIGN_CYCLES: u64 = 23_000_000_000;

pub fn validate_caller() -> Result<(), PythiaError> {
    let controllers = STATE.with(|s| s.borrow().controllers.clone());

    if controllers.contains(&ic_cdk::caller()) {
        return Ok(());
    }

    Err(PythiaError::NotAController)
}

pub async fn rec_eth_addr(msg: &str, sig: &str) -> Result<H160> {
    let siwe_canister = STATE
        .with(|s| s.borrow().siwe_canister)
        .expect("canister should be initialized");

    let msg = msg.to_string();
    let sig = sig.to_string();

    let (signer,): (String,) = ic_cdk::call(siwe_canister, "get_signer", (msg, sig))
        .await
        .map_err(|(code, msg)| anyhow!("{:?}: {}", code, msg))?;

    H160::from_str(&signer).context("failed to parse signer address")
}

pub async fn check_balance(exec_addr: &H160, chain: &Chain) -> Result<bool> {
    let balance = get_balance(exec_addr, &chain.rpc).await?;

    if balance < chain.min_balance {
        return Ok(false);
    }

    Ok(true)
}

pub async fn get_balance(address: &H160, rpc: &str) -> Result<Nat> {
    let w3 = Web3::new(ICHttp::new(rpc, None).context("failed to connect to a node")?);

    let balance = w3
        .eth()
        .balance(*address, None)
        .await
        .context("failed to get balance")?;

    Ok(u256_to_nat(balance))
}

#[inline]
pub fn add_brackets(data: &str) -> String {
    format!("[{}]", data)
}

pub fn cast_to_param_type(value: u64, kind: &str) -> Option<Token> {
    if kind == "bytes" {
        return Some(Token::Bytes(value.to_le_bytes().to_vec()));
    }
    if kind.contains("bytes") {
        return Some(Token::FixedBytes(value.to_le_bytes().to_vec()));
    }
    if kind.contains("uint") {
        return Some(Token::Uint(value.into()));
    }
    if kind.contains("int") {
        return Some(Token::Int(value.into()));
    }
    if kind.contains("string") {
        return Some(Token::String(value.to_string()));
    }

    None
}

pub async fn collect_fee(pub_key: &H160, exec_addr: &H160, chain: &Chain) -> Result<()> {
    let fee = STATE.with(|s| s.borrow().tx_fee.clone());
    if fee == 0 {
        return Ok(());
    }

    let w3 =
        Web3::new(ICHttp::new(chain.rpc.as_str(), None).context("failed to connect to a node")?);

    let key_info = KeyInfo {
        derivation_path: vec![pub_key.as_bytes().to_vec()],
        key_name: STATE.with(|s| s.borrow().key_name.clone()),
        ecdsa_sign_cycles: Some(ECDSA_SIGN_CYCLES),
    };

    let nonce = w3
        .eth()
        .transaction_count(*exec_addr, None)
        .await
        .context("failed to get nonce")?;
    let gas_price = w3
        .eth()
        .gas_price()
        .await
        .context("failed to get gas price")?;
    let gas_price = (gas_price / 10) * 13;

    let treasurer = H160::from_str(&chain.treasurer)
        .expect("should be valid the treasurer eth address");

    let tx = TransactionParameters {
        to: Some(treasurer),
        nonce: Some(nonce),
        value: nat_to_u256(&fee),
        gas_price: Some(gas_price),
        gas: ETH_TRANSFER_GAS_LIMIT.into(),
        ..Default::default()
    };

    let signed_tx = w3
        .accounts()
        .sign_transaction(
            tx,
            exec_addr.to_string(),
            key_info,
            nat_to_u64(&chain.chain_id),
        )
        .await?;

    for _ in 1..ATTEMPTS_TO_SEND_TX {
        match send_collect_fee_tx(w3.clone(), signed_tx.raw_transaction.clone()).await {
            Ok(_) => return Ok(()),
            Err(_) => continue,
        }
    }

    Ok(())
}

async fn send_collect_fee_tx(w3: Web3<ICHttp>, raw_transaction: Bytes) -> Result<()> {
    let tx_hash = w3
        .eth()
        .send_raw_transaction(raw_transaction.clone())
        .await?;

    wait_until_confimation(&tx_hash, &w3).await
}

pub fn nat_to_u64(nat: &Nat) -> u64 {
    let nat_digits = nat.0.to_u64_digits();
    let mut number: u64 = 0;
    if !nat_digits.is_empty() {
        number = *nat_digits.last().expect("nat should be a number");
    }
    number
}

pub fn nat_to_u256(nat: &Nat) -> U256 {
    U256::from_big_endian(&nat.0.to_bytes_be())
}

pub fn u256_to_nat(u256: U256) -> Nat {
    let mut buf: Vec<u8> = vec![];

    u256.to_big_endian(&mut buf);

    Nat(BigUint::from_bytes_be(&buf))
}
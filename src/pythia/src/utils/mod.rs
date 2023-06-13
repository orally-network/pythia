pub mod sybil;
pub mod multicall;
pub mod macros;

use std::str::FromStr;

use anyhow::{anyhow, Context, Result};

use candid::Nat;
use ic_web3::{
    ethabi::Token,
    ic::{KeyInfo, get_eth_addr},
    transports::ICHttp,
    types::{H160, U256},
    Web3,
};
use num_bigint::BigUint;

use crate::{
    types::errors::PythiaError, Chain, STATE, clone_with_state, update_state,
};

const ECDSA_SIGN_CYCLES: u64 = 23_000_000_000;

pub fn validate_caller() -> Result<(), PythiaError> {
    let controllers = clone_with_state!(controllers);
    if controllers.contains(&ic_cdk::caller()) {
        return Ok(());
    }

    Err(PythiaError::NotAController)
}

pub async fn rec_eth_addr(msg: &str, sig: &str) -> Result<H160> {
    let siwe_canister = clone_with_state!(siwe_canister)
        .expect("canister should be initialized");

    let msg = msg.to_string();
    let sig = sig.to_string();

    let (signer,): (String,) = ic_cdk::call(siwe_canister, "get_signer", (msg, sig))
        .await
        .map_err(|(code, msg)| anyhow!("{:?}: {}", code, msg))?;

    H160::from_str(&signer).context("failed to parse signer address")
}

pub fn check_balance(addr: &H160, chain: &Chain) -> bool {
    let balance = STATE.with(|state| {
        state
            .borrow()
            .balances
            .get(&chain.chain_id)
            .expect("chain should be initialized")
            .get(&hex::encode(addr.as_bytes()))
            .map(|balance| balance.amount.clone())
    });

    if let Some(balance) = balance {
        if balance < chain.min_balance {
            return false;
        }
        true
    } else {
        false
    }
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

    for i in u256.0.iter().rev().map(|e| *e).collect::<Vec<u64>>() {
        buf.extend(i.to_be_bytes());
    }

    Nat(BigUint::from_bytes_be(&buf))
}

pub async fn get_pma() -> Result<String> {
    if let Some(pma) = clone_with_state!(pma) {
        return Ok(pma);
    }

    let addr = get_eth_addr(None, Some(vec![vec![]]), clone_with_state!(key_name))
        .await
        .map(|addr| hex::encode(addr.as_bytes()))
        .map_err(|e| anyhow!("failed to get canister eth address: {e}"))?;
    
    update_state!(pma, Some(addr.clone()));

    Ok(addr)
}

pub fn get_key_info() -> KeyInfo {
    KeyInfo {
        derivation_path: vec![vec![]],
        key_name: clone_with_state!(key_name),
        ecdsa_sign_cycles: Some(ECDSA_SIGN_CYCLES),
    }
}

pub fn is_valid_eth_address(address: &str) -> bool {
    H160::from_str(address).is_ok()
}

pub fn check_subs_limit(pub_key: &H160) -> Result<()> {
    let owner = hex::encode(pub_key.as_bytes());
    
    STATE.with(|state| {
        let state = state.borrow();

        let owners = state
            .subscriptions
            .iter()
            .map(|(_, subs)| {
                subs
                    .iter()
                    .map(|sub| sub.owner.clone())
                    .collect::<Vec<String>>()
            })
            .fold(Vec::<String>::new(), |mut result, owners| {
                result.extend(owners);
                result
            });
        
        if owners.len() as u64 > state.subs_limit_total {
            return Err(anyhow!("total subscriptions limit reached"));
        }

        if owners.iter().filter(|&_owner| _owner.clone() == owner).count() as u64 > state.subs_limit_wallet {
            return Err(anyhow!("wallet subscriptions limit reached"));
        }

        Ok(())
    })
}

pub fn get_chain(chain_id: &Nat) -> Result<Chain> {
    STATE.with(|state| {
        Ok(state
            .borrow()
            .chains
            .get(chain_id)
            .ok_or(PythiaError::ChainDoesNotExist)?
            .clone())
    })
}

pub fn get_web3(chain_id: &Nat) -> Result<Web3<ICHttp>> {
    Ok(Web3::new(ICHttp::new(&get_chain(chain_id)?.rpc, None)?))
}

pub async fn get_gas_price(chain_id: &Nat) -> Result<Nat> {
    let gas_price = get_web3(chain_id)?
        .eth()
        .gas_price()
        .await
        .context("failed to get gas price")?;

    Ok(u256_to_nat(gas_price))
}

#[cfg(test)]
mod tests {
    use super::{nat_to_u256, u256_to_nat};

    #[test]
    fn convertable() {
        let u256 = 1234567890u64.into();
        println!("u256: {u256}");
        let nat = u256_to_nat(u256);
        println!("u256: {nat}");
        let u256 = nat_to_u256(&nat);
        println!("u256: {u256}");

        assert_eq!(u256, 1234567890u64.into());
    }
}
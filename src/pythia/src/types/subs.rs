use std::{str::FromStr, time::Duration};

use anyhow::{anyhow, Context, Result};
use serde_json::json;

use ic_cdk::export::{
    candid::{CandidType, Nat},
    serde::{Deserialize, Serialize},
};
use ic_cdk_timers::{set_timer_interval, set_timer};
use ic_web3::{
    contract::{Contract, Options},
    ethabi::Contract as EthabiContract,
    ethabi::Function,
    transports::ICHttp,
    types::H160,
    Web3,
};

use crate::{
    utils::{add_brackets, publish::{publish, get_input}, sybil::is_pair_exists},
    Chain, User, U256, USERS,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sub {
    pub id: u64,
    pub pair_id: Option<String>,
    pub chain_id: U256,
    pub contract_addr: H160,
    pub method: Method,
    pub frequency: u64,
    pub timer_id: String,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub enum MethodType {
    Pair,
    Random(String),
    Empty,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Method {
    pub name: String,
    pub abi: String,
    pub gas_limit: U256,
    pub method_type: MethodType
}

impl Sub {
    pub async fn instance(
        chain: &Chain,
        pair_id: Option<String>,
        contract_addr: &str,
        method_abi: &str,
        frequency: &u64,
        user: &User,
        is_random: bool,
    ) -> Result<Self> {
        let (method_abi, method_type) = resolve_abi(
            method_abi.into(),
            pair_id.clone(),
            is_random,
        )?;

        if let Some(pair_id) = pair_id.clone() {
            if !is_pair_exists(&pair_id).await? {
                return Err(anyhow!("pair id does not exist"));
            }
        }

        let id = USERS.with(|users| {
            users
                .borrow()
                .get(&user.pub_key)
                .expect("user should exist")
                .subs
                .len() as u64
        });

        let contract_addr =
            H160::from_str(contract_addr).context("failed to parse contract address")?;

        let owner = user.pub_key;

        let func: Function =
            serde_json::from_str(&method_abi).context("failed to parse method abi")?;

        let gas_limit = calculate_gas_limit(
            chain,
            &func.name,
            &method_type,
            &method_abi,
            &contract_addr,
            user,
            pair_id.clone(),
        )
        .await?;

        let method = Method {
            name: func.name,
            gas_limit,
            abi: method_abi.to_string(),
            method_type,
        };

        set_timer(Duration::from_secs(5), move || {
            publish(id, owner);
        });

        let timer_id = set_timer_interval(Duration::from_secs(*frequency), move || {
            publish(id, owner);
        });

        let timer_id = serde_json::to_string(&timer_id)?;

        Ok(Self {
            id,
            pair_id,
            chain_id: chain.chain_id,
            contract_addr,
            method,
            frequency: *frequency,
            timer_id,
        })
    }
}

fn resolve_abi(method_abi: String, pair_id: Option<String>, is_random: bool) -> Result<(String, MethodType)> {
    let raw_abi: Vec<&str> = method_abi.split_terminator(&['(', ')', ',',]).collect();

    if pair_id.is_some() {
        get_pair_abi(&raw_abi)
    } else if is_random {
        get_random_abi(&raw_abi)
    } else {
        get_empty_abi(&raw_abi)
    }
}

fn get_pair_abi(raw_abi: &[&str]) -> Result<(String, MethodType)> {
    let func_name = raw_abi
        .first()
        .ok_or(anyhow!("invalid method abi: a function name"))?
        .to_string();

    if raw_abi.len() != 5
        || raw_abi[1] != "string"
        || raw_abi[2] != " uint256"
        || raw_abi[3] != " uint256"
        || raw_abi[4] != " uint256" {
            return Err(anyhow!("invalid method abi: parameter types"));
    }

    let data = json!({
        "inputs": [
            {
                "internalType": "string",
                "name": "pair_id",
                "type": "string",
            },
            {
                "internalType": "uint256",
                "name": "price",
                "type": "uint256",
            },
            {
                "internalType": "uint256",
                "name": "decimals",
                "type": "uint256",
            },
            {
                "internalType": "uint256",
                "name": "timestamp",
                "type": "uint256",
            }
        ],
        "name": func_name,
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    });

    Ok((data.to_string(), MethodType::Pair))
}

fn get_random_abi(raw_abi: &[&str]) -> Result<(String, MethodType)> {
    if raw_abi.len() != 2 {
        return Err(anyhow!("invalid method abi: random should have one parameter"));
    }

    let func_name = raw_abi
        .first()
        .ok_or(anyhow!("invalid method abi: random, function name"))?
        .to_string();

    let param_type = raw_abi
        .get(1)
        .ok_or(anyhow!("invalid method abi: random, param type"))?
        .to_string();

    if !is_valid_func_param(&param_type) {
        return Err(anyhow!("invalid method abi, format"));
    }

    let data = json!({
        "inputs": [
            {
                "internalType": param_type,
                "name": "template",
                "type": param_type,
            }
        ],
        "name": func_name,
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    });

    Ok((data.to_string(), MethodType::Random(param_type)))
}

fn get_empty_abi(raw_abi: &[&str]) -> Result<(String, MethodType)> {
    if raw_abi.len() != 2 && raw_abi[1] != "" {
        return Err(anyhow!("invalid method abi: empty"));
    }

    let func_name = raw_abi
        .first()
        .ok_or(anyhow!("invalid method abi: empty, function name"))?
        .to_string();

    let data = json!({
        "inputs": [],
        "name": func_name,
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    });

    Ok((data.to_string(), MethodType::Empty))
}

async fn calculate_gas_limit(
    chain: &Chain,
    method_name: &str,
    method_type: &MethodType,
    method_abi: &str,
    contract_addr: &H160,
    user: &User,
    pair_id: Option<String>,
) -> Result<U256> {
    let w3 = Web3::new(
        ICHttp::new(chain.rpc.as_str(), None, None).context("failed to connect to a node")?,
    );

    let abi =
        EthabiContract::load(add_brackets(method_abi).as_bytes()).expect("abi should be valid");

    let contract = Contract::new(w3.eth(), *contract_addr, abi);

    let input = get_input(method_type, pair_id)
        .await?;

    let gas_limit = contract
        .estimate_gas(method_name, input, user.exec_addr, Options::default())
        .await?;

    // 1.2 multiplication
    Ok(U256((gas_limit / 5) * 6))
}

fn is_valid_func_param(func: &str) -> bool {
    match func {
        f if f.starts_with("string") 
            || f.starts_with("bytes") 
            || f.starts_with("uint") 
            || f.starts_with("int") => true,
        _ => false
    }
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct CandidSub {
    pub id: Nat,
    pub pair_id: Option<String>,
    pub chain_id: Nat,
    pub contract_addr: String,
    pub method_name: String,
    pub method_abi: String,
    pub method_type: MethodType,
    pub frequency: Nat,
}

impl From<Sub> for CandidSub {
    fn from(sub: Sub) -> Self {
        Self {
            id: Nat::from(sub.id),
            pair_id: sub.pair_id,
            chain_id: Nat::from(sub.chain_id),
            contract_addr: hex::encode(sub.contract_addr.as_bytes()),
            method_name: sub.method.name,
            method_abi: sub.method.abi,
            method_type: sub.method.method_type,
            frequency: Nat::from(sub.frequency),
        }
    }
}

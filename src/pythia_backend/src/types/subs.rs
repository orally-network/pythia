use std::{str::FromStr, time::Duration};

use anyhow::{anyhow, Context, Result};

use ic_cdk::export::candid::{CandidType, Nat};
use ic_cdk_timers::{set_timer_interval, TimerId};
use ic_web3::{
    contract::{Contract, Options},
    ethabi::Contract as EthabiContract,
    ethabi::{Function, ParamType},
    transports::ICHttp,
    types::H160,
    Web3,
};

use crate::{
    utils::{add_brackets, cast_to_param_type, publish::publish},
    Chain, PythiaError, User, U256, USERS,
};

#[derive(Clone, Debug)]
pub struct Sub {
    pub id: u64,
    pub chain_id: U256,
    pub contract_addr: H160,
    pub method: Method,
    pub frequency: u64,
    pub timer_id: TimerId,
    pub is_random: bool,
}

#[derive(Clone, Debug)]
pub struct Method {
    pub name: String,
    pub param: ParamType,
    pub abi: String,
    pub gas_limit: U256,
}

impl Sub {
    pub async fn instance(
        chain: &Chain,
        contract_addr: &str,
        method_abi: &str,
        frequency: &u64,
        user: &User,
        is_random: bool,
    ) -> Result<Self> {
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
        let timer_id = set_timer_interval(Duration::from_secs(*frequency), move || {
            publish(id, owner);
        });

        let func: Function =
            serde_json::from_str(method_abi).context("failed to parse method abi")?;

        let param = validate_params(&func)?;

        let gas_limit =
            calculate_gas_limit(chain, &func.name, &param, method_abi, &contract_addr, user)
                .await
                .context("failed to calculate gas limit")?;

        let method = Method {
            name: func.name,
            param,
            gas_limit,
            abi: method_abi.to_string(),
        };

        Ok(Self {
            id,
            chain_id: chain.chain_id,
            contract_addr,
            method,
            frequency: *frequency,
            timer_id,
            is_random,
        })
    }
}

async fn calculate_gas_limit(
    chain: &Chain,
    method_name: &str,
    param: &ParamType,
    method_abi: &str,
    contract_addr: &H160,
    user: &User,
) -> Result<U256> {
    let w3 = Web3::new(
        ICHttp::new(chain.rpc.as_str(), None, None).context("failed to connect to a node")?,
    );

    let abi =
        EthabiContract::load(add_brackets(method_abi).as_bytes()).expect("abi should be valid");

    let contract = Contract::new(w3.eth(), *contract_addr, abi);

    let input = cast_to_param_type(0, param).expect("should be able to cast");

    let gas_limit = contract
        .estimate_gas(method_name, input, user.exec_addr, Options::default())
        .await
        .context("failed to get gas_limit")?;

    // 1.2 multiplication
    Ok(U256((gas_limit / 5) * 6))
}

fn validate_params(func: &Function) -> Result<ParamType> {
    if func.inputs.len() != 1 {
        return Err(anyhow!(PythiaError::InvalidABIFunction(
            "inputs length should be 1".to_string()
        )));
    }

    let kind = func
        .inputs
        .first()
        .expect("a value should exists")
        .kind
        .clone();

    match kind {
        ParamType::Bytes => Ok(kind),
        ParamType::FixedBytes(_) => Ok(kind),
        ParamType::Uint(_) => Ok(kind),
        ParamType::Int(_) => Ok(kind),
        ParamType::String => Ok(kind),
        _ => Err(anyhow!(PythiaError::InvalidABIFunction(
            "input should be supported".to_string()
        ))),
    }
}

#[derive(Clone, Debug, CandidType)]
pub struct CandidSub {
    pub id: Nat,
    pub chain_id: Nat,
    pub contract_addr: String,
    pub method_name: String,
    pub method_abi: String,
    pub frequency: Nat,
    pub is_random: bool,
}

impl From<Sub> for CandidSub {
    fn from(sub: Sub) -> Self {
        Self {
            id: Nat(sub.id.into()),
            chain_id: Nat::from(sub.chain_id),
            contract_addr: sub.contract_addr.to_string(),
            method_name: sub.method.name,
            method_abi: sub.method.abi,
            frequency: Nat(sub.frequency.into()),
            is_random: sub.is_random,
        }
    }
}
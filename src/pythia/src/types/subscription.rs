use std::str::FromStr;

use candid::Nat;
use ic_cdk::export::{candid::CandidType, serde::{Deserialize, Serialize}};

use anyhow::{Result, anyhow, Context};
use ic_web3::{types::H160, ethabi::Function};
use serde_json::json;

use crate::{utils::{sybil::is_pair_exists, is_valid_eth_address, check_subs_limit}, STATE};

use super::methods::{Method, MethodType};

#[derive(Clone, Debug, Default, Serialize, Deserialize, CandidType)]
pub struct Subscription {
    pub id: Nat,
    pub owner: String,
    pub contract_addr: String,
    pub frequency: Nat,
    pub method: Method,
    pub status: SubscriptionStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, CandidType, Default)]
pub struct SubscriptionStatus {
    pub is_active: bool,
    pub last_update: Nat,
}

impl Subscription {
    pub fn builder() -> SubscriptionBuilder {
        SubscriptionBuilder::default()
    }
}

#[derive(Clone, Debug, Default)]
pub struct SubscriptionBuilder {
    owner: Option<String>,
    contract_addr: Option<String>,
    pair_id: Option<String>,
    gas_limit: Option<Nat>,
    frequency: Option<Nat>,
    abi: Option<String>,
    is_random: Option<bool>,
}

impl SubscriptionBuilder {
    pub fn owner(mut self, owner: &str) -> Self {
        self.owner = Some(owner.into());
        self
    }

    pub fn contract(mut self, contract_addr: &str) -> Self {
        self.contract_addr = Some(contract_addr.into());
        self
    }

    pub fn random(mut self, value: bool) -> Self {
        self.is_random = Some(value);
        self
    }

    pub fn pair(mut self, pair_id: Option<String>) -> Self {
        self.pair_id = pair_id;
        self
    }

    pub fn method(mut self, abi: &str, gas_limit: &Nat, frequency: &Nat) -> Self {
        self.abi = Some(abi.into());
        self.gas_limit = Some(gas_limit.clone());
        self.frequency = Some(frequency.clone());
        self
    }

    pub async fn build(&self) -> Result<Subscription> {
        let owner = self.owner.clone().context("owner is not set")?;
        let contract_addr = self.contract_addr.clone().context("contract address is not set")?;
        let abi = self.abi.clone().context("abi is not set")?;
        let gas_limit = self.gas_limit.clone().context("gas limit is not set")?;
        let is_random = self.is_random.context("is_random is not set")?;
        let frequency = self.frequency.clone().context("frequency is not set")?;

        if !is_valid_eth_address(&owner) {
            return Err(anyhow!("invalid ethereum address"));
        }

        if !is_valid_eth_address(&contract_addr) {
            return Err(anyhow!("invalid contract address"));
        }

        check_subs_limit(&H160::from_str(&owner)?)?;

        let (abi, method_type) = resolve_abi(abi, self.pair_id.clone(), is_random)?;
        if let Some(pair_id) = self.pair_id.clone() {
            if !is_pair_exists(&pair_id).await? {
                return Err(anyhow!("pair id does not exist"));
            }
        }

        let name = serde_json::from_str::<Function>(&abi)
            .context("failed to parse method abi")?
            .name;

        Ok(Subscription {
            id: get_new_subsciption_id(),
            owner,
            contract_addr,
            frequency,
            method: Method {
                name,
                abi,
                gas_limit,
                method_type,
            },
            status: SubscriptionStatus {
                is_active: true,
                ..Default::default()
            },
        })
    }

}

fn get_new_subsciption_id() -> Nat {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.subscriptions_index += 1;
        state.subscriptions_index.into()
    })
}

fn resolve_abi(
    method_abi: String,
    pair_id: Option<String>,
    is_random: bool,
) -> Result<(String, MethodType)> {
    let raw_abi: Vec<&str> = method_abi.split_terminator(&['(', ')', ',']).collect();

    if let Some(pair_id) = pair_id {
        get_pair_abi(&raw_abi, &pair_id)
    } else if is_random {
        get_random_abi(&raw_abi)
    } else {
        get_empty_abi(&raw_abi)
    }
}

fn get_pair_abi(raw_abi: &[&str], pair_id: &str) -> Result<(String, MethodType)> {
    let func_name = raw_abi
        .first()
        .ok_or(anyhow!("invalid method abi: a function name"))?
        .to_string();

    if raw_abi.len() != 5
        || raw_abi[1] != "string"
        || raw_abi[2] != " uint256"
        || raw_abi[3] != " uint256"
        || raw_abi[4] != " uint256"
    {
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

    Ok((data.to_string(), MethodType::Pair(pair_id.into())))
}

fn get_random_abi(raw_abi: &[&str]) -> Result<(String, MethodType)> {
    if raw_abi.len() != 2 {
        return Err(anyhow!(
            "invalid method abi: random should have one parameter"
        ));
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
    if raw_abi.len() != 2 && !raw_abi[1].is_empty() {
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

fn is_valid_func_param(func: &str) -> bool {
    matches!(func, f if f.starts_with("string") 
            || f.starts_with("bytes") 
            || f.starts_with("uint") 
            || f.starts_with("int"))
}
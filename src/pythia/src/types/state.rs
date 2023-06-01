use std::collections::HashMap;

use candid::Principal;
use ic_cdk::export::{candid::CandidType, serde::{Deserialize, Serialize}};

use super::{subs::Sub, chains::Chain, U256};

#[derive(Debug, Clone, Serialize, Deserialize, CandidType, Default)]
pub struct State {
    pub controllers: Vec<Principal>,
    pub chains: HashMap<U256, Chain>,
    pub tx_fee: U256,
    pub key_name: String,
    pub siwe_canister: Option<Principal>,
    pub sybil_canister: Option<Principal>,
    pub subs_limit_wallet: u64,
    pub subs_limit_total: u64,
    pub subs: Vec<Sub>,
    pub exec_addrs: HashMap<String, String>,
}
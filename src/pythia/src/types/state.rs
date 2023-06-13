use std::collections::HashMap;

use candid::{Nat, Principal};
use ic_cdk::export::{candid::CandidType, serde::{Deserialize, Serialize}};

use super::{chains::Chain, balance::UserBalance, withdraw::WithdrawRequest, subscription::Subscription};

#[derive(Debug, Clone, Serialize, Deserialize, CandidType, Default)]
pub struct State {
    pub controllers: Vec<Principal>,
    pub chains: HashMap<Nat, Chain>,
    pub tx_fee: Nat,
    pub key_name: String,
    pub siwe_canister: Option<Principal>,
    pub sybil_canister: Option<Principal>,
    pub subs_limit_wallet: u64,
    pub subs_limit_total: u64,
    pub pma: Option<String>,
    /// chain id => user's public key => PUB (Pythia User Balance)
    pub balances: HashMap<Nat, HashMap<String, UserBalance>>,
    /// chain id => withdraw requests
    pub withdraw_requests: HashMap<Nat, Vec<WithdrawRequest>>,
    /// chain id => subscriptions
    pub subscriptions: HashMap<Nat, Vec<Subscription>>,
    pub timer_frequency: u64,
    pub subscriptions_index: u64,
    pub is_timer_active: bool,
}
use candid::{Nat, Principal};
use ic_cdk::export::{
    candid::CandidType,
    serde::{Deserialize, Serialize},
};

use super::{
    balance::Balances, chains::Chains, subscription::{Subscriptions, SubscriptionsIndexer}, whitelist::Whitelist,
    withdraw::WithdrawRequests,
};

#[derive(Debug, Clone, Serialize, Deserialize, CandidType, Default)]
pub struct State {
    pub initialized: bool,
    pub controllers: Vec<Principal>,
    pub chains: Chains,
    pub tx_fee: Nat,
    pub key_name: String,
    pub siwe_canister: Option<Principal>,
    pub sybil_canister: Option<Principal>,
    pub subs_limit_wallet: Nat,
    pub subs_limit_total: Nat,
    pub pma: Option<String>,
    pub balances: Balances,
    pub withdraw_requests: WithdrawRequests,
    pub subscriptions: Subscriptions,
    pub timer_frequency: Nat,
    pub subscriptions_indexer: SubscriptionsIndexer,
    pub is_timer_active: bool,
    pub whitelist: Whitelist,
}

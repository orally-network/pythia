use candid::{CandidType, Nat, Principal};
use serde::{Deserialize, Serialize};

use super::{
    balance::Balances,
    chains::Chains,
    subscription::{Subscriptions, SubscriptionsIndexer},
    timer::Timer,
    whitelist::Whitelist,
    withdraw::WithdrawRequests,
};

#[derive(Debug, Clone, Serialize, Deserialize, CandidType, Default)]
pub struct State {
    #[deprecated]
    pub initialized: bool,
    #[deprecated]
    pub controllers: Vec<Principal>,
    pub chains: Chains,
    pub tx_fee: Nat,
    pub key_name: String,
    pub sybil_canister: Option<Principal>,
    pub subs_limit_wallet: Nat,
    pub subs_limit_total: Nat,
    pub pma: Option<String>,
    pub balances: Balances,
    pub withdraw_requests: WithdrawRequests,
    pub subscriptions: Subscriptions,
    pub timer_frequency: Nat,
    pub subscriptions_indexer: SubscriptionsIndexer,
    #[deprecated]
    pub is_timer_active: bool,
    pub timer: Option<Timer>,
    pub whitelist: Whitelist,
}

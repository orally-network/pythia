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

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct State {
    #[deprecated]
    pub initialized: bool,
    #[deprecated]
    pub controllers: Vec<Principal>,
    pub chains: Chains,
    pub tx_fee: Nat,
    pub key_name: String,
    pub siwe_canister: Option<Principal>,
    pub sybil_canister: Option<Principal>,
    pub ic_eth_rpc_canister: Principal,
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

impl Default for State {
    #[allow(deprecated)]
    fn default() -> Self {
        Self {
            initialized: false,
            chains: Chains::default(),
            tx_fee: 0.into(),
            key_name: "".to_string(),
            siwe_canister: None,
            sybil_canister: None,
            ic_eth_rpc_canister: Principal::from_text("6yxaq-riaaa-aaaap-abkpa-cai").unwrap(),
            subs_limit_wallet: 5.into(),
            subs_limit_total: 100.into(),
            pma: None,
            balances: Balances::default(),
            withdraw_requests: WithdrawRequests::default(),
            subscriptions: Subscriptions::default(),
            timer_frequency: (5 * 60).into(),
            subscriptions_indexer: SubscriptionsIndexer::default(),
            timer: None,
            whitelist: Whitelist::default(),
            controllers: vec![],
            is_timer_active: false,
        }
    }
}

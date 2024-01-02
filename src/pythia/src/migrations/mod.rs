use std::{collections::HashMap, time::Duration};

use crate::{
    jobs::publisher,
    log, metrics,
    types::{
        balance::Balances,
        chains::Chains,
        methods::{ExecutionCondition, Method, MethodType},
        subscription::{Subscription, SubscriptionStatus, Subscriptions, SubscriptionsIndexer},
        timer::Timer,
        whitelist::Whitelist,
        withdraw::WithdrawRequests,
    },
    utils::{
        canister::set_custom_panic_hook,
        metrics::{Metric, Metrics, METRICS},
    },
    State, STATE,
};
use candid::{CandidType, Nat, Principal};
use ic_cdk::{post_upgrade, pre_upgrade, storage};
use ic_cdk_timers::set_timer;
use ic_utils::{logger, monitor};
use serde::{Deserialize, Serialize};

const OLD_MULTICALL_CONTRACT_ADDRESS: &str = "0x88e33D0d7f9d130c85687FC73655457204E29467";

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, Default)]
pub enum OldMethodType {
    Pair(String),
    Random(String),
    #[default]
    Empty,
}

impl From<OldMethodType> for MethodType {
    fn from(old_method_type: OldMethodType) -> Self {
        match old_method_type {
            OldMethodType::Pair(pair) => MethodType::Feed(pair),
            OldMethodType::Random(random) => MethodType::Random(random),
            OldMethodType::Empty => MethodType::Empty,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, CandidType, Default)]
pub struct OldMethod {
    pub name: String,
    pub abi: String,
    pub gas_limit: Nat,
    pub chain_id: Nat,
    pub method_type: OldMethodType,
    pub exec_condition: Option<ExecutionCondition>,
}

impl From<OldMethod> for Method {
    fn from(old_method: OldMethod) -> Self {
        Method {
            name: old_method.name,
            abi: old_method.abi,
            gas_limit: old_method.gas_limit,
            chain_id: old_method.chain_id,
            method_type: old_method.method_type.into(),
            exec_condition: old_method.exec_condition,
        }
    }
}


#[derive(Clone, Debug, Default, Serialize, Deserialize, CandidType)]
pub struct OldSubscription {
    pub id: Nat,
    pub label: Option<String>,
    pub owner: String,
    pub contract_addr: String,
    pub frequency: Option<Nat>,
    pub method: OldMethod,
    pub status: SubscriptionStatus,
}

impl From<OldSubscription> for Subscription {
    fn from(mut old_subscription: OldSubscription) -> Self {
        if let Some(ref freq) = old_subscription.frequency {
            old_subscription.method.exec_condition =
                Some(ExecutionCondition::Frequency(freq.clone()));
        }

        if old_subscription.frequency.is_none() && old_subscription.method.exec_condition.is_none()
        {
            log!("old subscription should have frequency or exec_condition");

            old_subscription.method.exec_condition =
                Some(ExecutionCondition::Frequency(Nat::from(3600)));
        }

        let new = Subscription {
            id: old_subscription.id,
            label: old_subscription.label.unwrap_or("label".to_string()),
            owner: old_subscription.owner,
            contract_addr: old_subscription.contract_addr,
            method: old_subscription.method.into(),
            status: old_subscription.status,
        };

        new
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, CandidType)]
pub struct OldSubscriptions(pub HashMap<Nat, Vec<OldSubscription>>);

impl From<OldSubscriptions> for Subscriptions {
    fn from(old_subscriptions: OldSubscriptions) -> Self {
        Subscriptions(
            old_subscriptions
                .0
                .into_iter()
                .map(|(chain_id, subs)| (chain_id, subs.into_iter().map(|s| s.into()).collect()))
                .collect(),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType, Default)]
pub struct OldState {
    #[deprecated]
    pub initialized: bool,
    #[deprecated]
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
    pub subscriptions: OldSubscriptions,
    pub timer_frequency: Nat,
    pub subscriptions_indexer: SubscriptionsIndexer,
    #[deprecated]
    pub is_timer_active: bool,
    pub timer: Option<Timer>,
    pub whitelist: Whitelist,
}

impl From<OldState> for State {
    fn from(old_state: OldState) -> Self {
        State {
            initialized: old_state.initialized,
            chains: old_state.chains,
            tx_fee: old_state.tx_fee,
            key_name: old_state.key_name,
            siwe_canister: old_state.siwe_canister,
            sybil_canister: old_state.sybil_canister,
            subs_limit_wallet: old_state.subs_limit_wallet,
            subs_limit_total: old_state.subs_limit_total,
            pma: old_state.pma,
            balances: old_state.balances,
            withdraw_requests: old_state.withdraw_requests,
            subscriptions: old_state.subscriptions.into(),
            timer_frequency: old_state.timer_frequency,
            subscriptions_indexer: old_state.subscriptions_indexer,
            timer: old_state.timer,
            whitelist: old_state.whitelist,
            controllers: old_state.controllers,
            is_timer_active: old_state.is_timer_active,
        }
    }
}

#[allow(non_snake_case)]
#[derive(CandidType, Clone, Debug, Default, Deserialize, Serialize)]
pub struct OldMetrics {
    pub ACTIVE_SUBSCRIPTIONS: Option<Metric>,
    pub RPC_OUTCALLS: Option<Metric>,
    pub ECDSA_SIGNS: Option<Metric>,
    pub SUCCESSFUL_RPC_OUTCALLS: Option<Metric>,
    pub SYBIL_OUTCALLS: Option<Metric>,
    pub SUCCESSFUL_SYBIL_OUTCALLS: Option<Metric>,
    pub CYCLES: Option<Metric>,
}

impl From<OldMetrics> for Metrics {
    fn from(value: OldMetrics) -> Self {
        Metrics {
            ACTIVE_SUBSCRIPTIONS: value.ACTIVE_SUBSCRIPTIONS.unwrap_or_default(),
            RPC_OUTCALLS: value.RPC_OUTCALLS.unwrap_or_default(),
            ECDSA_SIGNS: value.ECDSA_SIGNS.unwrap_or_default(),
            SUCCESSFUL_RPC_OUTCALLS: value.SUCCESSFUL_RPC_OUTCALLS.unwrap_or_default(),
            SYBIL_OUTCALLS: value.SYBIL_OUTCALLS.unwrap_or_default(),
            SUCCESSFUL_SYBIL_OUTCALLS: value.SUCCESSFUL_SYBIL_OUTCALLS.unwrap_or_default(),
            CYCLES: value.CYCLES.unwrap_or_default(),
        }
    }
}

#[pre_upgrade]
fn pre_upgrade() {
    let state = STATE.with(|state| state.take());

    let log_data = logger::pre_upgrade_stable_data();
    let monitor_data = monitor::pre_upgrade_stable_data();

    let metrics = METRICS.with(|metrics| metrics.take());

    storage::stable_save((state, log_data, monitor_data, metrics))
        .expect("should be valid canister data for pre upgrade");
}

#[post_upgrade]
fn post_upgrade() {
    #[allow(clippy::type_complexity)]
    let (mut state, log_data, monitor_data, metrics): (
        OldState,
        logger::PostUpgradeStableData,
        monitor::PostUpgradeStableData,
        Option<OldMetrics>,
    ) = storage::stable_restore().expect("should be valid canister data for post upgrade");

    logger::post_upgrade_stable_data(log_data);
    monitor::post_upgrade_stable_data(monitor_data);

    let timer_id = set_timer(Duration::from_secs(10), publisher::execute);
    let timer = Timer {
        id: serde_json::to_string(&timer_id).expect("should be valid timer id"),
        is_active: true,
    };

    state.timer = Some(timer);

    state.chains.0.iter_mut().for_each(|(_, chain)| {
        if chain.multicall_contract.is_none() {
            chain.multicall_contract = Some(OLD_MULTICALL_CONTRACT_ADDRESS.to_string());
        }
    });

    STATE.with(|s| s.replace(state.into()));
    if let Some(metrics) = metrics {
        METRICS.with(|m| m.replace(metrics.into()));

        STATE.with(|state| {
            let state = state.borrow();
            let subscriptions = &state.subscriptions;
            for (chain_id, subscriptions) in subscriptions.0.iter() {
                let mut active_subscriptions_count = 0;
                for subscription in subscriptions.iter() {
                    if subscription.status.is_active {
                        active_subscriptions_count += 1;
                    }
                }
                metrics!(set ACTIVE_SUBSCRIPTIONS, active_subscriptions_count as u128, chain_id.to_string());
            }
        });
    }

    set_custom_panic_hook();

    log!("post upgrade finished");
}

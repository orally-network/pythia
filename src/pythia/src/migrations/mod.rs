use std::{collections::HashMap, time::Duration};

use crate::{
    jobs::publisher,
    log,
    types::{
        balance::Balances,
        chains::Chains,
        methods::{ExecutionCondition, Method},
        subscription::{Subscription, SubscriptionStatus, Subscriptions, SubscriptionsIndexer},
        timer::Timer,
        whitelist::Whitelist,
        withdraw::WithdrawRequests,
    },
    utils::{
        canister::set_custom_panic_hook,
        metrics::{Metrics, METRICS},
    },
    State, STATE,
};
use candid::{CandidType, Nat, Principal};
use ic_cdk::{post_upgrade, pre_upgrade, storage};
use ic_cdk_timers::set_timer;
use ic_utils::{logger, monitor};
use serde::{Deserialize, Serialize};

const OLD_MULTICALL_CONTRACT_ADDRESS: &str = "0x88e33D0d7f9d130c85687FC73655457204E29467";

#[derive(Clone, Debug, Default, Serialize, Deserialize, CandidType)]
pub struct OldSubscription {
    pub id: Nat,
    pub owner: String,
    pub contract_addr: String,
    pub frequency: Option<Nat>,
    pub method: Method,
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
            owner: old_subscription.owner,
            contract_addr: old_subscription.contract_addr,
            method: old_subscription.method,
            status: old_subscription.status,
        };

        new
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, CandidType)]
pub struct OldSubscriptions(pub HashMap<Nat, Vec<OldSubscription>>);

impl From<OldSubscriptions> for Subscriptions {
    fn from(old_subscriptions: OldSubscriptions) -> Self {
        let mut subscriptions = Subscriptions::default();

        for (chain_id, old_subscriptions) in old_subscriptions.0 {
            for old_subscription in old_subscriptions {
                subscriptions
                    .0
                    .entry(chain_id.clone())
                    .or_default()
                    .push(old_subscription.into());
            }
        }

        subscriptions
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
        Option<Metrics>,
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
        METRICS.with(|m| m.replace(metrics));
    }

    set_custom_panic_hook();

    log!("post upgrade finished");
}

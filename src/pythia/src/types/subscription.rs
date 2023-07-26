use std::collections::HashMap;

use itertools::Itertools;

use futures::future::join_all;
use ic_cdk::export::{
    candid::{CandidType, Nat},
    serde::{Deserialize, Serialize},
};
use ic_web3_rs::ethabi::Function;

use anyhow::{Context, Error, Result};

use super::{errors::PythiaError, logger::PUBLISHER, methods::Method};
use crate::{
    log,
    utils::{abi, address, nat, sybil, time, validator, web3, canister},
    STATE,
};

const SUBSCRIPTIONS_FAILURES_LIMIT: u64 = 5;

#[derive(Clone, Debug, Default, Serialize, Deserialize, CandidType)]
pub struct SubscriptionsIndexer(Nat);

impl SubscriptionsIndexer {
    pub fn new_index() -> Nat {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.subscriptions_indexer.0 += 1;
            state.subscriptions_indexer.0.clone()
        })
    }
}

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
    pub executions_counter: Nat,
    pub failures_counter: Option<Nat>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, CandidType)]
pub struct SubsribeRequest {
    pub chain_id: Nat,
    pub pair_id: Option<String>,
    pub contract_addr: String,
    pub method_abi: String,
    pub frequency: Nat,
    pub is_random: bool,
    pub gas_limit: Nat,
    pub msg: String,
    pub sig: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, CandidType)]
pub struct UpdateSubscriptionRequest {
    pub chain_id: Nat,
    pub id: Nat,
    pub pair_id: Option<String>,
    pub gas_limit: Option<Nat>,
    pub method_abi: Option<String>,
    pub contract_addr: Option<String>,
    pub frequency: Option<Nat>,
    pub is_random: Option<bool>,
    pub msg: String,
    pub sig: String,
}

/// Chain id => Subscriptions
#[derive(Clone, Debug, Default, Serialize, Deserialize, CandidType)]
pub struct Subscriptions(pub HashMap<Nat, Vec<Subscription>>);

impl Subscriptions {
    pub async fn add(req: &SubsribeRequest, owner: &str) -> Result<Nat> {
        validator::subscription_frequency(&req.frequency)
            .context(PythiaError::InvalidSubscriptionFrequency)?;
        let (abi, method_type) =
            abi::resolve_abi(req.method_abi.clone(), req.pair_id.clone(), req.is_random)?;
        if let Some(pair_id) = req.pair_id.clone() {
            if !sybil::is_pair_exists(&pair_id).await? {
                return Err(PythiaError::PairDoesNotExist.into());
            }
        }

        let name = serde_json::from_str::<Function>(&abi)
            .context(PythiaError::InvalidABIFunctionName)?
            .name;
        let id = SubscriptionsIndexer::new_index();

        let subscription = Subscription {
            id: id.clone(),
            owner: owner.into(),
            contract_addr: req.contract_addr.clone(),
            frequency: req.frequency.clone(),
            method: Method {
                name,
                abi,
                chain_id: req.chain_id.clone(),
                gas_limit: req.gas_limit.clone(),
                method_type,
            },
            status: SubscriptionStatus {
                is_active: true,
                ..Default::default()
            },
        };

        STATE.with(|state| {
            state
                .borrow_mut()
                .subscriptions
                .0
                .get_mut(&req.chain_id)
                .context(PythiaError::ChainDoesNotExist)?
                .push(subscription);

            Ok::<(), Error>(())
        })?;

        Ok(id)
    }

    pub fn get(chain_id: &Nat, id: &Nat) -> Result<Subscription> {
        let id = id.clone();
        STATE.with(|state| {
            Ok(state
                .borrow()
                .subscriptions
                .0
                .get(chain_id)
                .context(PythiaError::ChainDoesNotExist)?
                .iter()
                .find(|s| s.id == id)
                .context(PythiaError::SubscriptionDoesNotExist)?
                .clone())
        })
    }

    pub fn get_all(
        chain_id: Option<Nat>,
        ids: Vec<Nat>,
        owner: Option<String>,
    ) -> Vec<Subscription> {
        STATE.with(|state| {
            let state = state.borrow();
            let mut subscriptions = state.subscriptions.0.values().fold(
                Vec::<Subscription>::new(),
                |mut result, subs| {
                    result.extend(subs.clone());
                    result
                },
            );

            if let Some(chain_id) = chain_id {
                subscriptions = subscriptions
                    .into_iter()
                    .filter(|sub| sub.method.chain_id == chain_id)
                    .collect::<Vec<Subscription>>();
            }

            if let Some(owner) = owner {
                let owner = address::normalize(&owner).expect("should be valid address format");
                subscriptions = subscriptions
                    .into_iter()
                    .filter(|sub| sub.owner == owner)
                    .collect::<Vec<Subscription>>();
            }

            if !ids.is_empty() {
                subscriptions = subscriptions
                    .into_iter()
                    .filter(|sub| ids.contains(&sub.id))
                    .collect::<Vec<Subscription>>();
            }

            subscriptions
        })
    }

    pub fn remove(chain_id: &Nat, owner: &str, id: &Nat) -> Result<()> {
        let id = id.clone();
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let subscriptions = state
                .subscriptions
                .0
                .get_mut(chain_id)
                .context(PythiaError::ChainDoesNotExist)?;
            let index = subscriptions
                .iter()
                .position(|s| s.id == id && s.owner == owner)
                .context(PythiaError::SubscriptionDoesNotExist)?;
            subscriptions.remove(index);
            Ok(())
        })
    }

    pub fn remove_all(chain_id: Option<Nat>, ids: Vec<Nat>, owner: Option<String>) -> Result<()> {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            state
                .subscriptions
                .0
                .iter_mut()
                .for_each(|(_chain_id, subs)| {
                    if let Some(chain_id) = chain_id.clone() {
                        if chain_id != _chain_id.clone() {
                            return;
                        }
                    }

                    subs.retain(|sub| {
                        if let Some(owner) = owner.clone() {
                            if owner != sub.owner.clone() {
                                return true;
                            }
                        }

                        if ids.is_empty() || ids.contains(&sub.id) {
                            return false;
                        }

                        true
                    })
                });

            Ok(())
        })
    }

    pub fn stop(chain_id: &Nat, owner: &str, id: &Nat) -> Result<()> {
        let id = id.clone();
        STATE.with(|state| {
            state
                .borrow_mut()
                .subscriptions
                .0
                .get_mut(chain_id)
                .context(PythiaError::ChainDoesNotExist)?
                .iter_mut()
                .find(|s| s.id == id && s.owner == owner)
                .context(PythiaError::SubscriptionDoesNotExist)?
                .status
                .is_active = false;

            Ok(())
        })
    }

    pub fn stop_all(chain_id: Option<Nat>, ids: Vec<Nat>, owner: Option<String>) -> Result<()> {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            state
                .subscriptions
                .0
                .iter_mut()
                .for_each(|(_chain_id, subs)| {
                    if let Some(chain_id) = chain_id.clone() {
                        if chain_id != _chain_id.clone() {
                            return;
                        }
                    }

                    subs.iter_mut().for_each(|sub| {
                        if let Some(owner) = owner.clone() {
                            if owner != sub.owner.clone() {
                                return;
                            }
                        }

                        if ids.is_empty() || ids.contains(&sub.id) {
                            sub.status.is_active = false;
                        }
                    });
                });

            Ok(())
        })
    }

    pub fn start(chain_id: &Nat, owner: &str, id: &Nat) -> Result<()> {
        let id = id.clone();
        STATE.with(|state| {
            state
                .borrow_mut()
                .subscriptions
                .0
                .get_mut(chain_id)
                .context(PythiaError::ChainDoesNotExist)?
                .iter_mut()
                .find(|s| s.id == id && s.owner == owner)
                .context(PythiaError::SubscriptionDoesNotExist)?
                .status
                .is_active = true;

            Ok(())
        })
    }

    pub fn start_all(chain_id: Option<Nat>, ids: Vec<Nat>, owner: Option<String>) -> Result<()> {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            state
                .subscriptions
                .0
                .iter_mut()
                .for_each(|(_chain_id, subs)| {
                    if let Some(chain_id) = chain_id.clone() {
                        if chain_id != _chain_id.clone() {
                            return;
                        }
                    }

                    subs.iter_mut().for_each(|sub| {
                        if let Some(owner) = owner.clone() {
                            if owner != sub.owner.clone() {
                                return;
                            }
                        }

                        if ids.is_empty() || ids.contains(&sub.id) {
                            sub.status.is_active = true;
                        }
                    });
                });

            Ok(())
        })
    }

    pub fn update(req: &UpdateSubscriptionRequest, address: &str) -> Result<()> {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let subscription = state
                .subscriptions
                .0
                .get_mut(&req.chain_id)
                .context(PythiaError::ChainDoesNotExist)?
                .iter_mut()
                .find(|sub| sub.id == req.id && sub.owner == address)
                .context(PythiaError::SubscriptionDoesNotExist)?;

            if let Some(gas_limit) = req.gas_limit.clone() {
                subscription.method.gas_limit = gas_limit;
            }

            if let Some(contract_addr) = req.contract_addr.clone() {
                subscription.contract_addr = address::normalize(&contract_addr)
                    .context(PythiaError::InvalidAddressFormat)?;
            }

            if let Some(frequency) = req.frequency.clone() {
                validator::subscription_frequency(&frequency)
                    .context(PythiaError::InvalidSubscriptionFrequency)?;
                subscription.frequency = frequency;
            }

            if let (Some(method_abi), Some(is_random)) = (req.method_abi.clone(), req.is_random) {
                let (abi, method_type) =
                    abi::resolve_abi(method_abi, req.pair_id.clone(), is_random)?;
                subscription.method.abi = abi;
                subscription.method.method_type = method_type;
            }

            Ok(())
        })
    }

    pub fn check_limits(owner: &str) -> Result<()> {
        STATE.with(|state| {
            let state = state.borrow();

            let owners = state
                .subscriptions
                .0
                .values()
                .flat_map(|subs| {
                    subs.iter()
                        .map(|sub| sub.owner.clone())
                        .collect::<Vec<String>>()
                })
                .collect::<Vec<String>>();

            if owners.len() as u64 > state.subs_limit_total {
                return Err(PythiaError::TotalSubscriptionsLimitReached.into());
            }

            if owners
                .iter()
                .filter(|&_owner| _owner.clone() == owner)
                .count() as u64
                > state.subs_limit_wallet
            {
                return Err(PythiaError::WalletSubscriptionsLimitReached.into());
            }

            Ok(())
        })
    }

    pub async fn stop_insufficients() -> Result<()> {
        let chains_to_check = STATE.with(|state| {
            state.borrow()
                .subscriptions
                .0
                .iter()
                .filter_map(|(chain_id, subs)| {
                    if subs.iter().any(|sub| sub.status.is_active) {
                        return Some(chain_id.clone());
                    }

                    None
                })
                .collect::<Vec<Nat>>()
        });

        let futures = chains_to_check.iter().map(|chain_id| web3::gas_price(chain_id)).collect::<Vec<_>>();
        let gas_prices = join_all(futures).await.into_iter().collect::<Result<Vec<Nat>>>()
            .context("failed to get gas prices")?;

        let futures = chains_to_check.iter().map(|chain_id| canister::fee(chain_id)).collect::<Vec<_>>();
        let fees = join_all(futures).await.into_iter().collect::<Result<Vec<Nat>>>()
            .context("failed to get fees")?;

        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let balances = state.balances.0.clone();
            let chains = state.chains.0.clone();
            let subscriptions = &mut state.subscriptions.0;
            for (i, (chain_id, subs)) in subscriptions.iter_mut().enumerate() {
                if !chains_to_check.contains(chain_id) {
                    continue;
                }
                
                let gas_price = gas_prices.get(i).context("gas price not found")?;
                let fee = fees.get(i).context("fee not found")?;
                let chain_min_balance = &chains
                    .get(chain_id)
                    .context(PythiaError::ChainDoesNotExist)?
                    .min_balance;

                for (owner, subs) in &subs.iter_mut().group_by(|sub| sub.owner.clone()) {
                    let subs = subs.collect::<Vec<&mut Subscription>>();
                    let balance = balances
                        .get(chain_id)
                        .context(PythiaError::ChainDoesNotExist)?
                        .get(&owner)
                        .context(PythiaError::UnableToGetBalance)?;

                    let mut need_funds = subs.iter().fold(Nat::from(0), |res, sub| res + (sub.method.gas_limit.clone() * gas_price.clone()) + fee.clone());
                    need_funds += chain_min_balance.clone();
                
                    if balance.amount < need_funds {
                        subs.into_iter().for_each(|sub| sub.status.is_active = false);
                    }
                }
            }

            Ok(())
        })
    }

    pub fn update_last_update(chain_id: &Nat, sub_id: &Nat, is_failed: bool) {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let subscription = state
                .subscriptions
                .0
                .get_mut(chain_id)
                .expect("chain should exist")
                .iter_mut()
                .find(|sub| sub.id == *sub_id)
                .expect("sub should exist");

            subscription.status.last_update += subscription.frequency.clone();
            subscription.status.executions_counter += 1;
            if is_failed {
                if let Some(failures_counter) = subscription.status.failures_counter.as_mut() {
                    *failures_counter += 1;

                    if nat::to_u64(failures_counter) >= SUBSCRIPTIONS_FAILURES_LIMIT {
                        subscription.status.is_active = false;
                        log!("[{PUBLISHER}] subscription {sub_id} on chain {chain_id} has reached failures limit, stopping it");
                    }
                } else {
                    subscription.status.failures_counter = Some(1.into());
                }
            }
        })
    }

    pub fn get_publishable() -> (Vec<(Nat, Vec<Subscription>)>, bool) {
        let mut is_active = false;
        STATE.with(|state| {
            let state = state.borrow();
            let prepared_subs = state
                .subscriptions
                .0
                .iter()
                .map(|(chain_id, subs)| {
                    (
                        chain_id.clone(),
                        subs.iter()
                            .filter(|sub| {
                                if !sub.status.is_active {
                                    return false;
                                }
                                is_active = true;

                                (sub.status.last_update.clone() + sub.frequency.clone())
                                    <= time::in_seconds()
                            })
                            .cloned()
                            .collect::<Vec<Subscription>>(),
                    )
                })
                .collect::<Vec<(Nat, Vec<Subscription>)>>();

            (prepared_subs, is_active)
        })
    }

    pub fn init_new_chain(chain_id: &Nat) -> Result<()> {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            if state.subscriptions.0.contains_key(chain_id) {
                return Err(PythiaError::ChainAlreadyExists.into());
            }
            state.subscriptions.0.insert(chain_id.clone(), vec![]);
            Ok(())
        })
    }

    pub fn deinit_chain(chain_id: &Nat) -> Result<()> {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            state
                .subscriptions
                .0
                .remove(chain_id)
                .context(PythiaError::ChainDoesNotExist)?;
            Ok(())
        })
    }
}

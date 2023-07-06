use candid::Nat;
use anyhow::Result;
use std::collections::HashMap;

use crate::{STATE, log, types::logger::PUBLISHER, types::subscription::Subscription};

pub fn execute() {
    ic_cdk::spawn(async {
        if let Err(e) = _execute() {
            log!("[{PUBLISHER}] error while executing publisher job: {e:?}");
        }
    })
}

fn _execute() -> Result<()> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        for (_, subscriptions) in state.subscriptions.0.iter_mut() {
            group_subscriptions(subscriptions);
        }
        Ok(())
    })
}

fn group_subscriptions(subscriptions: &mut Vec<Subscription>) {
    let mut frequency_map = HashMap::new();
    for subscription in subscriptions.iter_mut() {
        frequency_map.entry(subscription.frequency.clone()).or_insert(Vec::new()).push(subscription);
    }

    for (_, group) in frequency_map.iter_mut() {
        if group.len() > 1 {
            let max_last_update = group
                .iter()
                .map(|sub| sub.status.last_update.clone())
                .max()
                .unwrap_or(Nat::from(0));
            for subscription in group {
                subscription.status.last_update = max_last_update.clone();
            }
        }
    }
}

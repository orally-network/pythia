use anyhow::Result;
use std::collections::HashMap;

const LAST_UPDATE_ALIGNMENT: u64 = 5 * 60; // in seconds

use crate::{
    log,
    types::subscription::Subscription,
    types::{logger::PUBLISHER, methods::ExecutionCondition},
    STATE,
};

#[allow(dead_code)]
pub fn execute() {
    ic_cdk::spawn(async {
        if let Err(e) = group() {
            log!("[{PUBLISHER}] error while executing publisher job: {e:?}");
        }
    })
}

pub fn group() -> Result<()> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        for (_, subscriptions) in state.subscriptions.0.iter_mut() {
            group_subscriptions(subscriptions);
        }

        log!("[SUBSCRIPTIONS GROUPER] grouped");

        Ok(())
    })
}

fn group_subscriptions(subscriptions: &mut [Subscription]) {
    let mut frequency_map = HashMap::new();
    for subscription in subscriptions.iter_mut() {
        if let Some(exec_condition) = &subscription.method.exec_condition {
            if let ExecutionCondition::Frequency(frequency) = exec_condition {
                frequency_map
                    .entry(frequency.clone())
                    .or_insert(Vec::new())
                    .push(subscription);
            }
        }
    }

    for (_, group) in frequency_map.iter_mut() {
        group.iter_mut().for_each(|sub| {
            sub.status.last_update =
                (sub.status.last_update.clone() / LAST_UPDATE_ALIGNMENT) * LAST_UPDATE_ALIGNMENT;
        });
    }
}

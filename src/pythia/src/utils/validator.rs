use anyhow::Result;
use candid::Nat;

use crate::{clone_with_state, PythiaError};

pub fn subscription_frequency(frequency: &Nat) -> Result<()> {
    if *frequency < clone_with_state!(timer_frequency) {
        return Err(PythiaError::TimerFrequencyIsGreaterThanSubscriptionFrequency.into());
    }

    if (frequency.clone() % clone_with_state!(timer_frequency)) != 0 {
        return Err(PythiaError::TimerFrequencyIsNotDivisibleBySubscriptionFrequency.into());
    }

    Ok(())
}

pub fn caller() -> Result<()> {
    if !clone_with_state!(initialized) {
        return Err(PythiaError::ControllersWereNotInitialized.into());
    }

    let controllers = clone_with_state!(controllers);
    if controllers.contains(&ic_cdk::caller()) {
        return Ok(());
    }

    Err(PythiaError::NotAController.into())
}

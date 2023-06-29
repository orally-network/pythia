use anyhow::Result;
use candid::Nat;
use ic_cdk::api::is_controller;

use crate::{clone_with_state, PythiaError};

pub fn subscription_frequency(frequency: &Nat) -> Result<()> {
    #[allow(clippy::cmp_owned)]
    if frequency.clone() < Nat::from(60 * 30) {
        return Err(PythiaError::SubscriptionFrequencyIsTooLow.into());
    }

    if frequency.clone() < clone_with_state!(timer_frequency) {
        return Err(PythiaError::TimerFrequencyIsGreaterThanSubscriptionFrequency.into());
    }

    if (frequency.clone() % clone_with_state!(timer_frequency)) != 0 {
        return Err(PythiaError::TimerFrequencyIsNotDivisibleBySubscriptionFrequency.into());
    }

    Ok(())
}

pub fn caller() -> Result<()> {
    if is_controller(&ic_cdk::caller()) {
        return Ok(());
    }

    Err(PythiaError::NotAController.into())
}

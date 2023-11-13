use anyhow::Result;
use candid::Nat;
use ic_cdk::api::is_controller;

use crate::PythiaError;

pub fn subscription_frequency(frequency: u64, timer_frequency: Nat) -> Result<()> {
    #[allow(clippy::cmp_owned)]
    if frequency.clone() < Nat::from(60) {
        return Err(PythiaError::SubscriptionFrequencyIsTooLow.into());
    }

    if frequency.clone() < timer_frequency {
        return Err(PythiaError::TimerFrequencyIsGreaterThanSubscriptionFrequency.into());
    }

    if (frequency.clone() % timer_frequency) != 0 {
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

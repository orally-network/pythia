use anyhow::anyhow;
use candid::{CandidType, Nat};
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    clone_with_state, log,
    utils::{nat, sybil, time},
};

use super::subscription::Subscriptions;

#[derive(Error, Debug)]
pub enum ExecutionConditionError {
    #[error("frequency is too low")]
    FrequencyIsTooLow,
    #[error("frequency lower than the timer frequency")]
    FrequencyLowerThanTimerFrequency,
    #[error("invalid frequency: {0}")]
    InvalidFrequency(String),
    #[error(
        "frequency ({frequency}) is not multipliable by the timer frequency ({timer_frequency})"
    )]
    FrequencyIsNotMultipliableByTheTimerFrequency {
        frequency: Nat,
        timer_frequency: Nat,
    },
    #[error("change rate should be greater than or equil 1 and lower than 100")]
    InvalidChangeRate,
    #[error("Pair does not exist")]
    PairDoesNotExist,
    #[error("error: {0}")]
    Error(#[from] anyhow::Error),
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize, Default, Eq, PartialEq)]
pub enum PriceMutationType {
    Increase,
    Decrease,
    #[default]
    Both,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize, Eq, PartialEq)]
pub enum ExecutionCondition {
    Frequency(Nat),
    PriceMutation {
        mutation_rate: i64,
        pair_id: String,
        creation_price: u64,
        price_mutation_type: PriceMutationType,
    },
}

impl Default for ExecutionCondition {
    fn default() -> Self {
        ExecutionCondition::Frequency(Nat::from(60 * 30))
    }
}

impl ExecutionCondition {
    pub async fn check(
        &mut self,
        chain_id: &Nat,
        subscription_id: &Nat,
    ) -> Result<bool, ExecutionConditionError> {
        match self {
            ExecutionCondition::Frequency(_) => self.check_frequency(chain_id, subscription_id),
            ExecutionCondition::PriceMutation { .. } => self.check_price_mutation().await,
        }
    }

    fn check_frequency(
        &self,
        chain_id: &Nat,
        subscription_id: &Nat,
    ) -> Result<bool, ExecutionConditionError> {
        let ExecutionCondition::Frequency(frequency) = self else {
            return Ok(false);
        };

        let subscription_status = Subscriptions::get(chain_id, subscription_id)?.status;
        if time::in_seconds()
            > (nat::to_u64(&subscription_status.last_update) + nat::to_u64(frequency))
        {
            return Ok(true);
        }

        Ok(false)
    }

    async fn check_price_mutation(&mut self) -> Result<bool, ExecutionConditionError> {
        let ExecutionCondition::PriceMutation {
            mutation_rate: change_rate,
            pair_id,
            creation_price,
            price_mutation_type,
        } = self
        else {
            return Ok(false);
        };

        log!("creation rate: {}", creation_price);
        let rate = sybil::get_asset_data(pair_id).await?;
        log!("current rate: {}", rate.rate);
        let current_mutation_rate = BigInt::from(100)
            - ((BigInt::from(rate.rate) * BigInt::from(100)) / BigInt::from(*creation_price));
        log!("mutation rate: {}", change_rate);
        log!("current mutation rate: {}", current_mutation_rate);
        *creation_price = rate.rate;
        match price_mutation_type {
            PriceMutationType::Increase if current_mutation_rate >= BigInt::from(*change_rate) => {
                Ok(true)
            }
            PriceMutationType::Decrease if current_mutation_rate <= BigInt::from(*change_rate) => {
                Ok(true)
            }
            PriceMutationType::Both
                if current_mutation_rate.magnitude() >= BigInt::from(*change_rate).magnitude() =>
            {
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub async fn validate(&mut self) -> Result<(), ExecutionConditionError> {
        match self {
            ExecutionCondition::Frequency(_) => self.validate_frequency(),
            ExecutionCondition::PriceMutation { .. } => self.validate_price_mutation().await,
        }
    }

    fn validate_frequency(&self) -> Result<(), ExecutionConditionError> {
        let ExecutionCondition::Frequency(frequency) = self else {
            Err(anyhow!("execution condition is not frequency"))?
        };

        if nat::to_u64(frequency) < 60 {
            return Err(ExecutionConditionError::FrequencyIsTooLow);
        }

        if nat::to_u64(frequency) < clone_with_state!(timer_frequency) {
            return Err(ExecutionConditionError::FrequencyLowerThanTimerFrequency);
        }

        if (nat::to_u64(frequency) % clone_with_state!(timer_frequency)) != 0 {
            return Err(
                ExecutionConditionError::FrequencyIsNotMultipliableByTheTimerFrequency {
                    frequency: frequency.clone(),
                    timer_frequency: clone_with_state!(timer_frequency),
                },
            );
        }

        Ok(())
    }

    async fn validate_price_mutation(&mut self) -> Result<(), ExecutionConditionError> {
        let ExecutionCondition::PriceMutation {
            mutation_rate,
            pair_id,
            creation_price,
            ..
        } = self
        else {
            Err(anyhow!("execution condition is not price mutation"))?
        };

        if *mutation_rate < 1 || *mutation_rate >= 100 {
            return Err(ExecutionConditionError::InvalidChangeRate);
        }

        if !sybil::is_pair_exists(pair_id).await? {
            return Err(ExecutionConditionError::PairDoesNotExist);
        }

        let rate = sybil::get_asset_data(pair_id).await?;
        *creation_price = rate.rate;

        Ok(())
    }
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, Default)]
pub enum MethodType {
    Pair(String),
    Random(String),
    #[default]
    Empty,
}

#[derive(Clone, Debug, Serialize, Deserialize, CandidType, Default)]
pub struct Method {
    pub name: String,
    pub abi: String,
    pub gas_limit: Nat,
    pub chain_id: Nat,
    pub method_type: MethodType,
    pub exec_condition: Option<ExecutionCondition>,
}

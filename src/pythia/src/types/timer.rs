use anyhow::{Result, Context};

use ic_cdk::export::{
    candid::CandidType,
    serde::{Deserialize, Serialize},
};
use ic_cdk_timers::{TimerId, clear_timer};

use crate::{STATE, PythiaError};

#[derive(Clone, Debug, Default, Serialize, Deserialize, CandidType)]
pub struct Timer {
    pub id: String,
    pub is_active: bool,
}

impl Timer {
    pub fn update(id: TimerId) -> Result<()> {
        let id = serde_json::to_string(&id)?;
        STATE.with(|state| {
            let mut state = state.borrow_mut();

            let old_timer = state
                .timer
                .clone()
                .context(PythiaError::TimerIsNotInitialized)?;

            let new_timer = Timer {
                id,
                is_active: old_timer.is_active,  
            };

            state
                .timer = Some(new_timer);

            Ok(())
        })
    }

    pub fn activate() -> Result<()> {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let old_timer = state
                .timer
                .clone()
                .context(PythiaError::TimerIsNotInitialized)?;

            let new_timer = Timer {
                id: old_timer.id,
                is_active: true,
            };

            state
                .timer = Some(new_timer);

            Ok(())
        })
    }

    pub fn deactivate() -> Result<()> {
        STATE.with(|state| {
            let mut state = state.borrow_mut();

            let old_timer = state
                .timer
                .clone()
                .context(PythiaError::TimerIsNotInitialized)?;

            let new_timer = Timer {
                id: old_timer.id,
                is_active: false,
            };

            let id = serde_json::from_str::<TimerId>(
                &state
                    .timer
                    .clone()
                    .context(PythiaError::TimerIsNotInitialized)?
                    .id
            )?;
            
            clear_timer(id);

            state
                .timer = Some(new_timer);

            Ok(())
        })
    }

    pub fn is_active() -> bool {
        STATE.with(|state| {
            let state = state.borrow();

            state
                .timer
                .clone()
                .expect("Timer is not initialized")
                .is_active
        })
    }
}
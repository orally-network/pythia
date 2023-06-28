use anyhow::{Context, Result};

use ic_cdk::export::candid::Nat;
use ic_cdk::{query, update};

use crate::{
    jobs::publisher,
    log,
    types::{
        balance::Balances,
        subscription::{Subscription, Subscriptions, SubsribeRequest, UpdateSubscriptionRequest},
        timer::Timer,
        whitelist,
    },
    utils::{siwe, validator},
    PythiaError,
};

/// Create a new subscriptions.
///
/// # Arguments
///
/// * `req` - The SubscribeRequest candid type.
///
/// # Returns
///
/// A result with a subscription id
#[update]
pub async fn subscribe(req: SubsribeRequest) -> Result<Nat, String> {
    _subscribe(req)
        .await
        .map_err(|e| format!("failed to subsribe: {e:?}"))
}

async fn _subscribe(req: SubsribeRequest) -> Result<Nat> {
    let address = siwe::recover(&req.msg, &req.sig)
        .await
        .context(PythiaError::UnableToRecoverAddress)?;
    if !whitelist::is_whitelisted(&address) {
        return Err(PythiaError::UserIsNotWhitelisted.into());
    }
    if !Balances::is_sufficient(&req.chain_id, &address)? {
        return Err(PythiaError::InsufficientBalance.into());
    }
    Subscriptions::check_limits(&address)?;

    let id = Subscriptions::add(&req, &address)
        .await
        .context(PythiaError::UnableToAddSubscription)?;

    if !Timer::is_active() {
        publisher::execute();
    }

    log!("[SUBSCRIPTIONS] added, id: {id}");
    Ok(id)
}

/// Get a subscriptions by owner if present
///
/// # Arguments
///
/// * `owner` - The owner of the subscription, can be omitted
///
/// # Returns
///
/// A vector of subscriptions
#[query]
pub fn get_subscriptions(owner: Option<String>) -> Vec<Subscription> {
    Subscriptions::get_all(None, vec![], owner)
}

/// Stop a subscription
///
/// # Arguments
///
/// * `chain_id` - Unique identifier of the chain, for example Ethereum Mainnet is 1
/// * `sub_id` - The subscription id
/// * `msg` - SIWE message, For more information, refer to the [SIWE message specification](https://eips.ethereum.org/EIPS/eip-4361)
/// * `sig` - SIWE signature, For more information, refer to the [SIWE message specification](https://eips.ethereum.org/EIPS/eip-4361)
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
pub async fn stop_subscription(
    chain_id: Nat,
    sub_id: Nat,
    msg: String,
    sig: String,
) -> Result<(), String> {
    _stop_subscription(chain_id, sub_id, msg, sig)
        .await
        .map_err(|e| format!("failed to stop a subscription: {e:?}"))
}

pub async fn _stop_subscription(
    chain_id: Nat,
    sub_id: Nat,
    msg: String,
    sig: String,
) -> Result<()> {
    let address = siwe::recover(&msg, &sig)
        .await
        .context(PythiaError::UnableToRecoverAddress)?;
    if !whitelist::is_whitelisted(&address) {
        return Err(PythiaError::UserIsNotWhitelisted.into());
    }

    Subscriptions::stop(&chain_id, &address, &sub_id)
        .context(PythiaError::UnableToStopSubscription)?;

    log!("[SUBSCRIPTIONS] stopped, id: {sub_id}");
    Ok(())
}

/// Start a subscription
///
/// # Arguments
///
/// * `chain_id` - Unique identifier of the chain, for example Ethereum Mainnet is 1
/// * `sub_id` - The subscription id
/// * `msg` - SIWE message, For more information, refer to the [SIWE message specification](https://eips.ethereum.org/EIPS/eip-4361)
/// * `sig` - SIWE signature, For more information, refer to the [SIWE message specification](https://eips.ethereum.org/EIPS/eip-4361)
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
pub async fn start_subscription(
    chain_id: Nat,
    sub_id: Nat,
    msg: String,
    sig: String,
) -> Result<(), String> {
    _start_subscription(chain_id, sub_id, msg, sig)
        .await
        .map_err(|e| format!("failed to start a subscription: {e:?}"))
}

pub async fn _start_subscription(
    chain_id: Nat,
    sub_id: Nat,
    msg: String,
    sig: String,
) -> Result<()> {
    let address = siwe::recover(&msg, &sig)
        .await
        .context(PythiaError::UnableToRecoverAddress)?;
    if !whitelist::is_whitelisted(&address) {
        return Err(PythiaError::UserIsNotWhitelisted.into());
    }
    if !Balances::is_sufficient(&chain_id, &address)? {
        return Err(PythiaError::InsufficientBalance.into());
    }

    Subscriptions::start(&chain_id, &address, &sub_id)
        .context(PythiaError::UnableToStartSubscription)?;

    if !Timer::is_active() {
        publisher::execute();
    }

    log!("[SUBSCRIPTIONS] started, id: {sub_id}");
    Ok(())
}

/// Update a subscription
///
/// # Arguments
///
/// * `req` - The UpdateSubscriptionRequest candid type.
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
pub async fn update_subscription(req: UpdateSubscriptionRequest) -> Result<(), String> {
    _update_subscription(req)
        .await
        .map_err(|e| format!("failed to update a subscription: {e:?}"))
}

async fn _update_subscription(req: UpdateSubscriptionRequest) -> Result<()> {
    let address = siwe::recover(&req.msg, &req.sig)
        .await
        .context(PythiaError::UnableToRecoverAddress)?;
    if !whitelist::is_whitelisted(&address) {
        return Err(PythiaError::UserIsNotWhitelisted.into());
    }

    Subscriptions::update(&req, &address).context(PythiaError::UnableToUpdateSubscription)?;

    log!("[SUBSCRIPTIONS] updated, id: {}", req.id);
    Ok(())
}

/// Stop all subscriptions
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
pub fn stop_subscriptions() -> Result<(), String> {
    _stop_subscriptions().map_err(|e| format!("failed to stop subscriptions: {e:?}"))
}

fn _stop_subscriptions() -> Result<()> {
    validator::caller()?;
    Subscriptions::stop_all(None, vec![], None).context(PythiaError::UnableToStopSubscriptions)?;

    log!("[SUBSCRIPTIONS] stopped all");
    Ok(())
}

/// Remove all subscriptions
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
pub fn remove_subscriptions() -> Result<(), String> {
    _remove_subscriptions().map_err(|e| format!("{e:?}"))
}

pub fn _remove_subscriptions() -> Result<()> {
    validator::caller()?;
    Subscriptions::remove_all(None, vec![], None)
        .context(PythiaError::UnableToRemoveSubscriptions)?;

    log!("[SUBSCRIPTIONS] removed all");
    Ok(())
}

/// Remove a subscription by id
///
/// # Arguments
///
/// * `id` - The subscription id
///
/// # Returns
///
/// Returns a result that can contain an error message
#[update]
pub fn remove_subscription(id: Nat) -> Result<(), String> {
    _remove_subscription(id).map_err(|e| format!("{e:?}"))
}

pub fn _remove_subscription(id: Nat) -> Result<()> {
    validator::caller()?;
    Subscriptions::remove_all(None, vec![id.clone()], None)
        .context(PythiaError::UnableToRemoveSubscriptions)?;

    log!("[SUBSCRIPTIONS] removed, id: {}", id);
    Ok(())
}

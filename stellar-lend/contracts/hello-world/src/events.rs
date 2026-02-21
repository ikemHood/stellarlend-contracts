//! # Events Module
//!
//! Defines standardized event types for the lending protocol and provides
//! convenience functions for publishing them to the Soroban ledger.
//!
//! All core operations (deposit, withdrawal, borrow, repay) emit events through
//! this module, enabling off-chain indexers and analytics services to track
//! protocol activity.

use soroban_sdk::{contracttype, Address, Env, Symbol};

/// A standardized event structure for all protocol actions.
///
/// This enum allows for a consistent event format across the protocol,
/// making it easier for off-chain services to consume and interpret events.
/// Using a single event type with a `Symbol` for the action is more gas-efficient
/// than publishing multiple distinct event types with string literals.
///
/// # Fields
/// * `event_type` - The type of event (e.g., "deposit", "withdrawal").
/// * `user` - The primary user address associated with the event.
/// * `asset` - The asset involved in the transaction (optional).
/// * `amount` - The amount of the asset.
/// * `timestamp` - The ledger timestamp of the event.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    Deposit(DepositEvent),
    Withdrawal(WithdrawalEvent),
    Borrow(BorrowEvent),
    Repay(RepayEvent),
}

/// Event data for a deposit action.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepositEvent {
    pub user: Address,
    pub asset: Option<Address>,
    pub amount: i128,
    pub timestamp: u64,
}

/// Event data for a withdrawal action.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawalEvent {
    pub user: Address,
    pub asset: Option<Address>,
    pub amount: i128,
    pub timestamp: u64,
}

/// Event data for a borrow action.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BorrowEvent {
    pub user: Address,
    pub asset: Option<Address>,
    pub amount: i128,
    pub timestamp: u64,
}

/// Event data for a repay action.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepayEvent {
    pub user: Address,
    pub asset: Option<Address>,
    pub amount: i128,
    pub timestamp: u64,
}

/// Event data for a liquidation action.
/// Publishes an event to the ledger.
///
/// # Arguments
/// * `env` - The Soroban environment.
/// * `event` - The event to be published.
fn log_event(env: &Env, event: Event) {
    let event_type = match &event {
        Event::Deposit(_) => Symbol::new(env, "deposit"),
        Event::Withdrawal(_) => Symbol::new(env, "withdrawal"),
        Event::Borrow(_) => Symbol::new(env, "borrow"),
        Event::Repay(_) => Symbol::new(env, "repay"),
    };
    env.events().publish((event_type,), event);
}

// Convenience functions for logging specific events

/// Publish a deposit event to the ledger.
pub fn log_deposit(env: &Env, event: DepositEvent) {
    log_event(env, Event::Deposit(event));
}

/// Publish a withdrawal event to the ledger.
pub fn log_withdrawal(env: &Env, event: WithdrawalEvent) {
    log_event(env, Event::Withdrawal(event));
}

/// Publish a borrow event to the ledger.
pub fn log_borrow(env: &Env, event: BorrowEvent) {
    log_event(env, Event::Borrow(event));
}

/// Publish a repay event to the ledger.
pub fn log_repay(env: &Env, event: RepayEvent) {
    log_event(env, Event::Repay(event));
}

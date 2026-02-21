use soroban_sdk::{contractevent, contracttype, Address, Env};

/// Types of operations that can be paused.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum PauseType {
    /// Pause all protocol operations
    All = 0,
    /// Pause deposit operations
    Deposit = 1,
    /// Pause borrow operations
    Borrow = 2,
    /// Pause repay operations
    Repay = 3,
    /// Pause withdraw operations
    Withdraw = 4,
    /// Pause liquidation operations
    Liquidation = 5,
}

/// Storage keys for pause states.
#[contracttype]
#[derive(Clone)]
pub enum PauseDataKey {
    /// Pause state for a specific operation type
    State(PauseType),
}

/// Event data emitted on pause state change.
#[contractevent]
#[derive(Clone, Debug)]
pub struct PauseEvent {
    /// Operation type affected
    pub pause_type: PauseType,
    /// New pause state
    pub paused: bool,
    /// Admin who performed the action
    pub admin: Address,
}

/// Set pause state for a specific operation type
///
/// # Arguments
/// * `env` - The contract environment
/// * `admin` - The admin address (must authorize)
/// * `pause_type` - The operation type to pause/unpause
/// * `paused` - True to pause, false to unpause
pub fn set_pause(env: &Env, admin: Address, pause_type: PauseType, paused: bool) {
    // Store the pause state
    env.storage()
        .persistent()
        .set(&PauseDataKey::State(pause_type), &paused);

    // Emit event
    PauseEvent {
        pause_type,
        paused,
        admin,
    }
    .publish(env);
}

/// Check if a specific operation is paused
///
/// An operation is considered paused if either its specific pause flag
/// is set or the global `All` pause flag is set.
///
/// # Arguments
/// * `env` - The contract environment
/// * `pause_type` - The operation type to check
///
/// # Returns
/// True if paused, false otherwise
pub fn is_paused(env: &Env, pause_type: PauseType) -> bool {
    // Check global pause first
    if env
        .storage()
        .persistent()
        .get(&PauseDataKey::State(PauseType::All))
        .unwrap_or(false)
    {
        return true;
    }

    // Check specific operation pause
    if pause_type != PauseType::All {
        return env
            .storage()
            .persistent()
            .get(&PauseDataKey::State(pause_type))
            .unwrap_or(false);
    }

    false
}
